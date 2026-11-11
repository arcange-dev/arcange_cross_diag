import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const [gitignore, capabilityText, tauriConfigText, updaterText, releaseScriptText, releaseWorkflowText, rustSource, signingPreflightText] = await Promise.all([
  readFile(new URL("../.gitignore", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/capabilities/default.json", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  readFile(new URL("../src/lib/appUpdater.ts", import.meta.url), "utf8"),
  readFile(new URL("./prepare-release.mjs", import.meta.url), "utf8"),
  readFile(new URL("../.github/workflows/release.yml", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8"),
  readFile(new URL("./validate-release-secrets.mjs", import.meta.url), "utf8"),
]);

const capabilities = JSON.parse(capabilityText);
const tauriConfig = JSON.parse(tauriConfigText);

for (const pattern of [".env", ".tauri/", "*.key", "*.pem", "*.p12", "*.pfx", "src-tauri/tauri.release.conf.json"]) {
  assert.ok(gitignore.split(/\r?\n/).includes(pattern), `Git ignore should protect ${pattern}`);
}
assert.ok(capabilities.permissions.includes("updater:default"), "Updater capability must be explicitly enabled");
assert.ok(capabilities.permissions.includes("process:allow-restart"), "Only restart capability should be used for updater relaunch");
assert.ok(!capabilities.permissions.some((permission) => permission.includes("shell:allow-execute")), "Frontend must not have unrestricted shell execution capability");
assert.match(updaterText, /import\s*\{\s*isTauri\s*\}/, "Updater calls must be gated to a Tauri desktop runtime");
assert.match(updaterText, /VITE_UPDATER_ENABLED\s*===\s*"true"/, "Updater must be disabled unless enabled at release build time");
assert.match(updaterText, /downloadAndInstall\(/, "Updates must use the signed updater install path");
assert.match(releaseScriptText, /PRIVATE KEY|BEGIN \./, "Release preparation must reject private key material in public key input");
assert.match(releaseWorkflowText, /TAURI_SIGNING_PRIVATE_KEY:\s*\$\{\{\s*secrets\./, "Signing private key must be sourced from Actions secrets");
assert.match(releaseWorkflowText, /validate:release-secrets/, "Release workflow must validate signing configuration before building");
assert.match(signingPreflightText, /TAURI_SIGNING_PRIVATE_KEY/, "Signing preflight must require the private updater key");
assert.doesNotMatch(signingPreflightText, /process\.stdout\.write\([^\n]*(?:privateKey|publicKey)/, "Signing preflight must not print key variables");
assert.match(tauriConfig.app.security.csp, /object-src 'none'/, "CSP must prohibit object embedding");
assert.match(tauriConfig.app.security.csp, /frame-ancestors 'none'/, "CSP must prohibit framing");
assert.equal(tauriConfig.app.security.freezePrototype, true, "JavaScript prototypes should be frozen by Tauri");
const readinessBody = rustSource.match(/fn collect_runtime_readiness\(\) -> RuntimeReadiness \{([\s\S]*?)\n\}/)?.[1] ?? "";
assert.ok(readinessBody, "Runtime readiness collector must exist");
assert.doesNotMatch(readinessBody, /Command::new|run_command\(/, "Runtime readiness must not execute optional diagnostic tools");

const ignoredDirs = new Set(["node_modules", "target", ".git", "dist", "gen"]);
const secretPattern = /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/;
const sourceExtensions = new Set([".ts", ".tsx", ".js", ".mjs", ".rs", ".toml", ".json", ".md", ".yml", ".yaml", ".css", ".html"]);
let scanned = 0;
async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isDirectory()) {
      if (!ignoredDirs.has(entry.name)) await walk(path.join(directory, entry.name));
      continue;
    }
    const ext = path.extname(entry.name).toLowerCase();
    if (!sourceExtensions.has(ext) || entry.name === "package-lock.json") continue;
    const fullPath = path.join(directory, entry.name);
    const content = await readFile(fullPath, "utf8");
    scanned += 1;
    assert.ok(!secretPattern.test(content), `Possible private key material found in ${path.relative(root, fullPath)}`);
  }
}
await walk(root);

process.stdout.write(`[OK] Security source audit scanned ${scanned} text files; no PEM private-key block was found.\n`);
process.stdout.write("[OK] Secret-file ignore rules, least-privilege Tauri capabilities, updater gating, signed release secrets and WebView security settings are present.\n");
