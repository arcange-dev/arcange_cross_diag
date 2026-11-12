import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const [packageJson, tauriConfig, cargoToml, desktopApi, rustSource, appSource, workflow, capabilities, updaterSource, releaseWorkflow, securityWorkflow, releaseScript] = await Promise.all([
  readFile(new URL("../package.json", import.meta.url), "utf8").then(JSON.parse),
  readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8").then(JSON.parse),
  readFile(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8"),
  readFile(new URL("../src/lib/desktop.ts", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8"),
  readFile(new URL("../src/App.tsx", import.meta.url), "utf8"),
  readFile(new URL("../.github/workflows/cross-platform-checks.yml", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/capabilities/default.json", import.meta.url), "utf8").then(JSON.parse),
  readFile(new URL("../src/lib/appUpdater.ts", import.meta.url), "utf8"),
  readFile(new URL("../.github/workflows/release.yml", import.meta.url), "utf8"),
  readFile(new URL("../.github/workflows/security.yml", import.meta.url), "utf8"),
  readFile(new URL("./prepare-release.mjs", import.meta.url), "utf8"),
]);

const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
assert.match(packageJson.version, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/, "package.json version must be valid SemVer");
assert.equal(tauriConfig.version, packageJson.version, "Tauri config version must match package.json");
assert.equal(cargoVersion, packageJson.version, "Cargo package version must match package.json");
assert.ok(packageJson.engines?.node?.startsWith(">=22"), "Node.js 22 or newer must be declared");
assert.ok(packageJson.scripts?.["test:security:source"], "Security source audit must be available as an npm script");
assert.ok(packageJson.scripts?.["test:bundle"], "Built platform bundles must be checked after packaging");
assert.ok(packageJson.scripts?.["test:storage-trends"], "Storage trend regression tests must be available");
assert.ok(packageJson.scripts?.["test:storage-history"], "SQLite storage-history migration tests must be available");
assert.ok(packageJson.scripts?.["test:command-output"], "Native command-output tests must be available");
assert.ok(packageJson.scripts?.["test:release-config"], "Updater release configuration tests must be available");
assert.ok(packageJson.scripts?.["test:first-pc:script"], "Portable first-PC preflight safety tests must be available");
assert.ok(packageJson.scripts?.["test:first-pc:windows"], "Windows first-PC preflight command must be available");
const firstPcScript = await readFile(new URL("./first-pc-test.ps1", import.meta.url), "utf8");
assert.match(firstPcScript, /Win32_OperatingSystem/);
assert.match(firstPcScript, /Win32_Processor/);
assert.match(firstPcScript, /Win32_ComputerSystem/);
assert.match(firstPcScript, /Win32_LogicalDisk/);
assert.doesNotMatch(firstPcScript, /\bsmartctl\s+-[a-z]/i, "First-PC preflight must not invoke SMART operations");
assert.doesNotMatch(firstPcScript, /\bsfc\s+\/scannow|\bDISM\s+\/Online\s+\/Cleanup-Image\s+\/RestoreHealth/i, "First-PC preflight must not run repair commands");
assert.doesNotMatch(firstPcScript, /\bwinget\s+upgrade\b|\bFormat-Volume\b|\bClear-Disk\b/i, "First-PC preflight must not install updates or write to disks");
assert.ok(packageJson.scripts?.["validate:release-secrets"], "Release signing secrets must be validated before publishing");

const invoked = new Set([...desktopApi.matchAll(/invoke(?:<[\s\S]*?>)?\(\s*"([a-z_]+)"/g)].map((match) => match[1]));
const handlerBody = rustSource.match(/tauri::generate_handler!\s*\[([\s\S]*?)\]/)?.[1];
assert.ok(handlerBody, "Rust Tauri command handler list must exist");
const registered = new Set([...handlerBody.matchAll(/\b([a-z][a-z0-9_]*)\s*,?/g)].map((match) => match[1]));
const missing = [...invoked].filter((name) => !registered.has(name));
assert.deepEqual(missing, [], `Frontend Tauri commands missing from Rust handler: ${missing.join(", ")}`);
assert.ok(invoked.has("get_runtime_readiness"), "Frontend must expose runtime readiness diagnostics");
assert.match(rustSource, /fn\s+collect_runtime_readiness/, "Rust runtime readiness collector must exist");
assert.match(appSource, /Runtime readiness/, "Settings must show the runtime readiness panel");
assert.match(appSource, /StorageTrendPanel/, "Storage page must show historical SMART trend comparison");
const trendsSource = await readFile(new URL("../src/lib/storageTrends.mjs", import.meta.url), "utf8");
assert.match(trendsSource, /Missing readings are unknown/, "Storage trends must not coerce missing readings to zero");
assert.match(trendsSource, /storageScansToCsv/, "Storage history CSV export must be available");
assert.match(trendsSource, /isMetricWorsening/, "Trend direction must distinguish health metrics from cumulative usage counters");
assert.match(trendsSource, /serial numbers are deliberately neither collected nor exported/, "Storage trend export must not include serial numbers");
assert.match(rustSource, /CREATE TABLE IF NOT EXISTS storage_scans/, "Storage history must use transactional SQLite storage");
assert.match(rustSource, /TransactionBehavior::Immediate/, "Legacy history migration must be serialized to avoid duplicate imports");
assert.match(rustSource, /storage_history_json_migrated/, "Legacy JSON storage history must be migrated only once");
assert.match(rustSource, /String::from_utf8_lossy/, "Native command output must tolerate non-UTF-8 device text");
assert.match(rustSource, /MAX_COMMAND_OUTPUT_BYTES:\s*usize\s*=\s*8\s*\*\s*1024\s*\*\s*1024/, "Command output must have a bounded in-memory capture size");
assert.match(rustSource, /Continue draining after the cap is reached/, "Oversized child output must still be drained to avoid pipe backpressure");
assert.match(rustSource, /MAX_LEGACY_HISTORY_BYTES:\s*u64\s*=\s*16\s*\*\s*1024\s*\*\s*1024/, "Legacy history migration must reject oversized files");
assert.match(rustSource, /oversized_legacy_storage_history_is_not_read_or_marked_as_migrated/, "Oversized legacy history must have a regression test");
assert.match(rustSource, /legacy_storage_history_migrates_latest_records_once_and_preserves_source_file/, "Legacy storage history migration must be regression-tested");
assert.match(rustSource, /command_output_reader_enforces_output_size_limit_and_drains_input/, "Command output limit must have a regression test");
const trendTests = await readFile(new URL("../tests/storage-trends.test.mjs", import.meta.url), "utf8");
assert.match(trendTests, /orders unix-prefixed Rust timestamps correctly/, "Storage trend tests must cover native Rust timestamp format");

assert.match(rustSource, /match\s+pass\s*%\s*6/, "RAM smoke test must use six deterministic patterns");
assert.match(rustSource, /for\s+pass\s+in\s+0\.\.6/, "RAM smoke test must execute six passes");
assert.match(appSource, /Six different data patterns/, "UI must describe the six-pattern RAM test accurately");
assert.match(rustSource, /Sha256::new\(\)/, "Recovery image hashing must use SHA-256");
assert.match(rustSource, /fn\s+collect_amd_gpu_telemetry/, "AMD telemetry adapter must be present");
assert.match(rustSource, /fn\s+collect_intel_gpu_telemetry/, "Intel telemetry adapter must be present");
assert.match(rustSource, /fn\s+pnp_problem_description/, "Windows Configuration Manager code mapping must be present");
assert.match(workflow, /ubuntu-24\.04[\s\S]*windows-latest|windows-latest[\s\S]*ubuntu-24\.04/, "CI must exercise Linux and Windows runners");
assert.match(workflow, /npm run build:desktop/, "CI must attempt desktop bundle creation");
assert.match(workflow, /npm run test:storage-history/, "CI must run SQLite storage-history migration tests");
assert.match(workflow, /npm run test:command-output/, "CI must run native command-output handling tests");

assert.ok(packageJson.dependencies["@tauri-apps/plugin-updater"], "Tauri updater frontend plugin must be present");
assert.ok(packageJson.dependencies["@tauri-apps/plugin-process"], "Tauri process frontend plugin must be present");
assert.match(cargoToml, /tauri-plugin-updater\s*=\s*"2(?:\.[0-9]+)?"/, "Rust updater plugin dependency must be present");
assert.match(cargoToml, /tauri-plugin-process\s*=\s*"2"/, "Rust process plugin dependency must be present");
assert.match(rustSource, /tauri_plugin_updater::Builder::new\(\)\.build\(\)/, "Rust updater plugin must be initialized");
assert.match(rustSource, /tauri_plugin_process::init\(\)/, "Rust process plugin must be initialized");
assert.ok(capabilities.permissions.includes("updater:default"), "Updater permissions must be explicitly scoped to the main window");
assert.ok(capabilities.permissions.includes("process:allow-restart"), "Only process restart permission should be explicitly scoped to the main window");
assert.match(updaterSource, /check\(/, "Updater UI must check for updates");
assert.match(updaterSource, /downloadAndInstall\(/, "Updater UI must use the signed updater install path");
assert.match(updaterSource, /relaunch\(/, "Updater UI must relaunch after a successful update");
assert.match(releaseWorkflow, /TAURI_SIGNING_PRIVATE_KEY:/, "Release workflow must use a GitHub secret for updater signing");
assert.match(releaseWorkflow, /TAURI_UPDATER_PUBLIC_KEY:/, "Release workflow must inject the updater public key as a repository variable");
assert.match(releaseWorkflow, /uploadUpdaterJson:\s*true/, "Release workflow must publish the Tauri updater manifest");
assert.match(releaseWorkflow, /prerelease:\s*\$\{\{[^\n]*contains\(github\.ref_name, '-rc\.'\)/, "Release candidates must be marked as prereleases");
assert.match(releaseWorkflow, /cargo test --manifest-path src-tauri\/Cargo.toml/, "Release workflow must run Rust tests before publishing");
assert.match(releaseWorkflow, /npm audit --audit-level=high/, "Release workflow must audit npm dependencies");
assert.match(releaseWorkflow, /cargo audit --deny warnings/, "Release workflow must audit Cargo dependencies");
assert.match(securityWorkflow, /npm audit/, "Security workflow must audit npm dependencies");
assert.match(securityWorkflow, /cargo audit/, "Security workflow must audit Rust dependencies");
assert.match(securityWorkflow, /codeql-action\/analyze/, "Security workflow must include CodeQL analysis");
assert.match(releaseScript, /TAURI_UPDATER_PUBLIC_KEY/, "Release preparation must require an updater public key");
assert.match(releaseScript, /updaterEndpointForRelease/, "Release preparation must select the updater endpoint by release channel");
const channelSource = await readFile(new URL("./release-channel.mjs", import.meta.url), "utf8");
assert.match(channelSource, /releases\/download\/.*latest\.json/, "Release candidates must use a tag-pinned updater manifest");
assert.match(channelSource, /releases\/latest\/download\/latest\.json/, "Stable releases must use the latest stable updater manifest");
assert.ok(packageJson.scripts["test:ci"].includes("test:storage-history") && packageJson.scripts["test:ci"].includes("test:command-output"), "Full CI test command must include SQLite migration and command-output tests");
assert.ok(packageJson.scripts["test:ci"].includes("test:release-config"), "Full CI test command must include updater release configuration tests");
assert.match(releaseWorkflow, /validate:release-secrets/, "Release workflow must validate signing secrets before preparing the updater config");
assert.match(releaseScript, /PRIVATE KEY/, "Release preparation must reject private-key material in the public-key variable");
assert.match(tauriConfig.app.security.csp, /object-src 'none'/, "CSP must prohibit plugin/object content");
assert.match(tauriConfig.app.security.csp, /frame-ancestors 'none'/, "CSP must prohibit framing");
assert.equal(tauriConfig.app.security.freezePrototype, true, "Tauri should freeze JavaScript prototypes");

process.stdout.write(`[OK] ${packageJson.version} package/Cargo/Tauri versions match.\n`);
process.stdout.write(`[OK] ${invoked.size} frontend Tauri command(s) are registered in Rust.\n`);
process.stdout.write("[OK] Hardware/recovery diagnostics, runtime readiness, transactional SQLite storage history, storage trends and Windows/Linux CI checks are present.\n");
process.stdout.write("[OK] Signed updater wiring, capability permissions, release preflight and dependency/security workflows are present.\n");
