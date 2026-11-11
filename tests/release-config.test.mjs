import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFile, rm } from "node:fs/promises";
import { updaterEndpointForRelease } from "../scripts/release-channel.mjs";

const prepareScript = new URL("../scripts/prepare-release.mjs", import.meta.url);
const secretsScript = new URL("../scripts/validate-release-secrets.mjs", import.meta.url);
const releasePreflightScript = new URL("../scripts/release-preflight.mjs", import.meta.url);
const generatedConfig = new URL("../src-tauri/tauri.release.conf.json", import.meta.url);
const baseEnv = { ...process.env, GITHUB_REPOSITORY: "arcange9/arcange-crossdiag" };
const samplePublicKey = "RWQ1bXktc2FtcGxlLXB1YmxpYy1rZXk=";
const samplePrivateKey = ["untrusted comment: sample private signing key", "RWQ1bXktc2FtcGxlLXByaXZhdGUta2V5"].join("\n");
const privateHeader = ["-----BEGIN ", "PRIVATE KEY----- secret"].join("");

function run(script, env) {
  return spawnSync(process.execPath, [script.pathname], { encoding: "utf8", env });
}

test("release configuration embeds only public key and HTTPS updater endpoint", async () => {
  try {
    const result = run(prepareScript, {
      ...baseEnv,
      TAURI_UPDATER_PUBLIC_KEY: samplePublicKey,
      RELEASE_TAG: "v1.0.0-rc.2",
    });
    assert.equal(result.status, 0, result.stderr);
    const config = JSON.parse(await readFile(generatedConfig, "utf8"));
    assert.equal(config.bundle.createUpdaterArtifacts, true);
    assert.equal(config.plugins.updater.pubkey, samplePublicKey);
    assert.deepEqual(config.plugins.updater.endpoints, [
      "https://github.com/arcange9/arcange-crossdiag/releases/download/v1.0.0-rc.2/latest.json",
    ]);
    assert.doesNotMatch(JSON.stringify(config), /sample private signing key/);
  } finally {
    await rm(generatedConfig, { force: true });
  }
});

test("release configuration generator rejects an absent public key", () => {
  const result = run(prepareScript, { ...baseEnv, TAURI_UPDATER_PUBLIC_KEY: "" });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /TAURI_UPDATER_PUBLIC_KEY is required/);
});

test("release configuration generator rejects private material in public variable", () => {
  const result = run(prepareScript, { ...baseEnv, TAURI_UPDATER_PUBLIC_KEY: privateHeader });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /private key was supplied where the updater public key is expected/i);
});

test("release signing preflight requires both public and private key values", () => {
  const result = run(secretsScript, {
    ...baseEnv,
    TAURI_UPDATER_PUBLIC_KEY: samplePublicKey,
    TAURI_SIGNING_PRIVATE_KEY: "",
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /TAURI_SIGNING_PRIVATE_KEY repository secret is missing/);
});

test("release signing preflight never prints key values", () => {
  const result = run(secretsScript, {
    ...baseEnv,
    TAURI_UPDATER_PUBLIC_KEY: samplePublicKey,
    TAURI_SIGNING_PRIVATE_KEY: samplePrivateKey,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.doesNotMatch(`${result.stdout}${result.stderr}`, /sample private signing key|RWQ1bXktc2FtcGxl/);
});


test("release preflight accepts the exact release-candidate tag", () => {
  const result = run(releasePreflightScript, { ...baseEnv, RELEASE_TAG: "v1.0.0-rc.2" });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /1\.0\.0-rc\.2/);
});

test("release preflight rejects a tag that does not match the candidate version", () => {
  const result = run(releasePreflightScript, { ...baseEnv, RELEASE_TAG: "v1.0.0" });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /must exactly match v1\.0\.0-rc\.2/);
});

test("signing preflight rejects malformed repository identity", () => {
  const result = run(secretsScript, {
    ...baseEnv,
    GITHUB_REPOSITORY: "not-a-repository",
    TAURI_UPDATER_PUBLIC_KEY: samplePublicKey,
    TAURI_SIGNING_PRIVATE_KEY: samplePrivateKey,
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /owner\/repository format/);
});


test("signing preflight rejects identical public and private key values", () => {
  const result = run(secretsScript, {
    ...baseEnv,
    TAURI_UPDATER_PUBLIC_KEY: samplePublicKey,
    TAURI_SIGNING_PRIVATE_KEY: samplePublicKey,
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /must not be identical/);
});


test("release configuration generator rejects a mismatched release tag", () => {
  const result = run(prepareScript, {
    ...baseEnv,
    TAURI_UPDATER_PUBLIC_KEY: samplePublicKey,
    RELEASE_TAG: "v1.0.0",
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /must exactly match v1\.0\.0-rc\.2/);
});


test("release channels pin prereleases but stable builds follow latest stable", () => {
  assert.equal(
    updaterEndpointForRelease("arcange9/arcange-crossdiag", "1.0.0-rc.2", "v1.0.0-rc.2"),
    "https://github.com/arcange9/arcange-crossdiag/releases/download/v1.0.0-rc.2/latest.json",
  );
  assert.equal(
    updaterEndpointForRelease("arcange9/arcange-crossdiag", "1.0.0", "v1.0.0"),
    "https://github.com/arcange9/arcange-crossdiag/releases/latest/download/latest.json",
  );
});
