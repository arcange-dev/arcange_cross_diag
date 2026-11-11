import { spawnSync } from "node:child_process";
import process from "node:process";

const platform = process.platform;
const isWindows = platform === "win32";
const isLinux = platform === "linux";

function probe(label, command, args = ["--version"], required = true) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    windowsHide: true,
    timeout: 5000,
  });

  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`.trim();
  const ok = !result.error && result.status === 0;
  const version = output.split(/\r?\n/).find((line) => line.trim()) ?? "Available";
  const marker = ok ? "[OK]" : required ? "[MISSING]" : "[OPTIONAL]";
  process.stdout.write(`${marker} ${label}${ok ? ` — ${version}` : ""}\n`);
  return ok;
}

function probeOptionalBinary(label, binary) {
  const lookup = isWindows ? "where.exe" : "which";
  const result = spawnSync(lookup, [binary], { encoding: "utf8", windowsHide: true, timeout: 3000 });
  const ok = !result.error && result.status === 0;
  const path = (result.stdout ?? "").split(/\r?\n/).find((line) => line.trim())?.trim();
  process.stdout.write(`${ok ? "[OK]" : "[OPTIONAL]"} ${label}${ok && path ? ` — ${path}` : ""}\n`);
  return ok;
}

process.stdout.write("\nArcange CrossDiag — environment doctor\n");
process.stdout.write("========================================\n");

const nodeOk = Number(process.versions.node.split(".")[0]) >= 22;
process.stdout.write(`${nodeOk ? "[OK]" : "[MISSING]"} Node.js ${process.versions.node} (22 or newer required)\n`);
const npmOk = isWindows
  ? probe("npm", "cmd.exe", ["/d", "/s", "/c", "npm --version"])
  : probe("npm", "npm", ["--version"]);
const rustOk = probe("Rust compiler", "rustc", ["--version"]);
const cargoOk = probe("Cargo", "cargo", ["--version"]);
const smartctlOk = probe("smartctl / smartmontools", "smartctl", ["--version"], false);
const gpuToolStatus = [
  ["NVIDIA-SMI", "nvidia-smi"],
  ["AMD SMI CLI", "amd-smi"],
  ["Intel GPU Top", "intel_gpu_top"],
].map(([label, binary]) => probeOptionalBinary(label, binary));
const pciUtilsOk = probeOptionalBinary("PCI inventory (lspci)", "lspci");
const lsblkOk = isLinux ? probeOptionalBinary("Removable media inventory (lsblk)", "lsblk") : false;

process.stdout.write("\nPlatform-specific notes\n");
if (isWindows) {
  process.stdout.write("- Install Microsoft C++ Build Tools and ensure the WebView2 Runtime is available.\n");
  process.stdout.write("- Install smartmontools with: winget install --id smartmontools.smartmontools -e\n");
  process.stdout.write("- NVIDIA telemetry requires a compatible NVIDIA driver package that provides nvidia-smi.\n");
} else if (isLinux) {
  process.stdout.write("- Install Tauri's GTK, WebKitGTK, AppIndicator, and build prerequisites for your distribution.\n");
  process.stdout.write("- Ubuntu/Debian smartmontools and PCI tools: sudo apt install smartmontools pciutils\n");
  process.stdout.write("- Intel GPU telemetry (where supported): sudo apt install intel-gpu-tools\n");
  process.stdout.write("- AMD GPU telemetry: follow the official AMD SMI/ROCm installation guide for your GPU and distribution.\n");
  process.stdout.write("- Fedora smartmontools: sudo dnf install smartmontools\n");
  process.stdout.write("- Arch smartmontools: sudo pacman -S smartmontools\n");
} else {
  process.stdout.write("- This release's automated CI currently targets Windows and Linux.\n");
}

process.stdout.write("\nNext steps\n");
if (nodeOk && npmOk && rustOk && cargoOk) {
  process.stdout.write("- Required Node and Rust tools are present. Run: npm install && npm run tauri dev\n");
} else {
  process.stdout.write("- Install the missing required tools, open a new terminal, then run: npm run doctor\n");
}
if (!smartctlOk) {
  process.stdout.write("- SMART storage scans remain unavailable until smartmontools is installed and smartctl is on PATH.\n");
}
if (!gpuToolStatus.some(Boolean)) {
  process.stdout.write("- No optional vendor GPU telemetry CLI was found. GPU inventory remains available; live vendor metrics may be unavailable.\n");
}
if (isLinux && !pciUtilsOk) process.stdout.write("- Install pciutils for richer PCI/driver binding inventory.\n");
if (isLinux && !lsblkOk) process.stdout.write("- Install util-linux/lsblk to inventory removable recovery media.\n");

if (!nodeOk || !npmOk || !rustOk || !cargoOk) process.exitCode = 1;
