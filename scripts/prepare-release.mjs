import { readFile, writeFile } from "node:fs/promises";
import process from "node:process";
import { updaterEndpointForRelease } from "./release-channel.mjs";

const publicKey = (process.env.TAURI_UPDATER_PUBLIC_KEY ?? "").trim();
const repository = (process.env.GITHUB_REPOSITORY ?? "").trim();
const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
const tag = (process.env.RELEASE_TAG ?? process.env.GITHUB_REF_NAME ?? `v${packageJson.version}`).trim();

if (!publicKey) {
  process.stderr.write("TAURI_UPDATER_PUBLIC_KEY is required. Add the Tauri updater public key as a GitHub Actions repository variable.\n");
  process.exit(1);
}
if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
  process.stderr.write("GITHUB_REPOSITORY must be in owner/repository format.\n");
  process.exit(1);
}
if (/PRIVATE KEY|BEGIN .*PRIVATE/i.test(publicKey)) {
  process.stderr.write("A private key was supplied where the updater public key is expected. Never put the private key in repository variables.\n");
  process.exit(1);
}
if (tag !== `v${packageJson.version}`) {
  process.stderr.write(`Release tag ${tag} must exactly match v${packageJson.version}.\n`);
  process.exit(1);
}

// GitHub's /releases/latest route intentionally excludes prereleases. Pin RC builds
// to their exact draft/published tag; stable builds follow the latest stable release.
const isReleaseCandidate = /-rc\./.test(packageJson.version);
const updateEndpoint = updaterEndpointForRelease(repository, packageJson.version, tag);

const config = {
  bundle: { createUpdaterArtifacts: true },
  plugins: {
    updater: {
      pubkey: publicKey,
      endpoints: [updateEndpoint],
      windows: { installMode: "basicUi" },
    },
  },
};

await writeFile(
  new URL("../src-tauri/tauri.release.conf.json", import.meta.url),
  `${JSON.stringify(config, null, 2)}\n`,
  { mode: 0o600 },
);
process.stdout.write(`Prepared signed updater build configuration for ${repository} (${tag}).\n`);
process.stdout.write(`Updater channel: ${isReleaseCandidate ? "tag-pinned release candidate" : "latest stable release"}.\n`);
process.stdout.write("The generated file contains only the updater public key; the signing private key is read separately by Tauri from TAURI_SIGNING_PRIVATE_KEY.\n");
