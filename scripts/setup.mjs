import { spawnSync } from "node:child_process";
import process from "node:process";

const platform = process.platform;
const npmCommand = platform === "win32" ? "npm.cmd" : "npm";
const rustCommand = (name) => spawnSync(name, ["--version"], { encoding: "utf8", windowsHide: true, timeout: 5000 });

function heading(text) {
  process.stdout.write(`\n${text}\n${"-".repeat(text.length)}\n`);
}

function commandAvailable(name) {
  const result = rustCommand(name);
  return !result.error && result.status === 0;
}

heading("Arcange CrossDiag — terminal setup");
const nodeMajor = Number(process.versions.node.split(".")[0]);
if (!Number.isFinite(nodeMajor) || nodeMajor < 22) {
  process.stderr.write(`Node.js 22 or newer is required. Current version: ${process.versions.node}\n`);
  process.exit(1);
}
process.stdout.write(`[OK] Node.js ${process.versions.node}\n`);

const npm = spawnSync(npmCommand, ["--version"], { encoding: "utf8", windowsHide: true, timeout: 5000, shell: platform === "win32" });
if (npm.error || npm.status !== 0) {
  process.stderr.write("npm was not found. Install Node.js from https://nodejs.org/ and reopen your terminal.\n");
  process.exit(1);
}
process.stdout.write(`[OK] npm ${String(npm.stdout || npm.stderr).trim().split(/\r?\n/)[0]}\n`);

heading("Installing frontend dependencies");
process.stdout.write("Running npm install. This downloads the React, TypeScript, Vite and Tauri CLI packages.\n\n");
const install = spawnSync(npmCommand, ["install", "--no-audit", "--no-fund"], {
  stdio: "inherit",
  windowsHide: true,
  timeout: 15 * 60 * 1000,
  shell: platform === "win32",
});
if (install.error || install.status !== 0) {
  process.stderr.write(`\nDependency installation failed${install.error ? `: ${install.error.message}` : ` (exit ${install.status})`}. Check your network/proxy settings and run npm install again.\n`);
  process.exit(1);
}

heading("Native desktop prerequisites");
const rustOk = commandAvailable("rustc");
const cargoOk = commandAvailable("cargo");
process.stdout.write(`${rustOk ? "[OK]" : "[MISSING]"} Rust compiler\n`);
process.stdout.write(`${cargoOk ? "[OK]" : "[MISSING]"} Cargo\n`);
if (!rustOk || !cargoOk) {
  process.stdout.write("Install the Rust stable toolchain from https://rustup.rs/, then reopen the terminal.\n");
}
process.stdout.write("Install the Tauri platform prerequisites for your operating system: https://v2.tauri.app/start/prerequisites/\n");
process.stdout.write("Optional storage support requires smartmontools: https://www.smartmontools.org/\n");

heading("Next commands");
process.stdout.write("npm run doctor       # verify local prerequisites\n");
process.stdout.write("npm run tauri dev    # start the desktop app\n");
process.stdout.write("npm run test:rust    # run native unit tests\n");
process.stdout.write("npm run build:desktop # build a platform-specific bundle\n");
process.stdout.write("\nSetup finished. No system settings, drivers, firmware or OS packages were changed.\n");
