# First test on your Windows PC — CrossDiag v1.0.0-rc.2

This is the first acceptance test for a real Windows computer. It verifies that the development environment can prepare and launch CrossDiag, then checks that the app reports basic facts correctly. Start with read-only inventory only. Do not begin with repairs, updates, stress tests, SMART scans or firmware operations.

## 1. Extract the correct release

1. Download `arcange-crossdiag-v1.0.0-rc.2.zip`.
2. Extract it to a short local path such as `C:\Dev\arcange-crossdiag-v1.0.0-rc.2` (avoid running from inside the ZIP or a protected `Program Files` folder).
3. Open the extracted folder. Confirm it contains `package.json`, `src`, and `src-tauri`.

## 2. Install the development prerequisites

Install these before attempting a desktop build:

- Node.js 22 or newer (npm 10 or newer).
- Rust stable 1.90 or newer via <https://rustup.rs/>. Use the MSVC toolchain on Windows.
- Microsoft Visual Studio Build Tools with **Desktop development with C++** and a Windows SDK.
- The Microsoft Edge WebView2 Runtime.
- The Windows prerequisites listed by Tauri: <https://v2.tauri.app/start/prerequisites/>.

After installation, close every PowerShell/Terminal window and open a new **normal (not Administrator)** PowerShell window.

## 3. Run the read-only PC preflight

From the extracted project root, you can run the hardware/toolchain preflight before installing any npm dependencies:

```powershell
npm run test:first-pc:windows
```

Or invoke it directly if you prefer not to use npm yet:

```powershell
.\scripts\first-pc-test.ps1
```

Then install frontend dependencies and run the remaining checks:

```powershell
npm install
npm run doctor
npm run test:first-pc:script
```

The PowerShell preflight writes a report in `test-results/first-pc-test-YYYYMMDD-HHMMSS.txt`.

Expected results:

- Project source files and package/Tauri/Cargo versions are present and consistent.
- Node.js 22+, npm, Rust and Cargo are detected.
- Windows edition/build and architecture are detected.
- Manufacturer/model, CPU, logical/physical core counts and installed RAM are reported where Windows exposes them.
- Fixed drive letters, capacities and free space are reported.
- Optional tools such as `smartctl` and `nvidia-smi` are marked available or missing; missing optional tools are warnings, not failures.
- The script reports whether PowerShell is elevated. The first app run should be as a standard user.

The script does not collect serial numbers, MAC addresses, usernames or recovery keys. It does not run SMART commands, stress tests, repairs, software updates, driver changes, firmware operations or disk writes.

## 4. Build and launch CrossDiag

Run these commands one by one so an error is easy to identify:

```powershell
npm run typecheck
npm run build
npm run test:rust
npm run tauri dev
```

If `npm run typecheck` fails, stop and save the complete error. If `npm run build` fails, stop before running Tauri. If Rust compilation fails, save the first compiler error and the lines immediately above it. Do not hide errors by rerunning with Administrator privileges.

## 5. The first actual in-app test: System Facts

When CrossDiag opens, it starts on **System overview** and automatically requests a system snapshot. If the dashboard is still waiting or shows an error, click **Collect system facts** (or **Refresh inventory**).

Check these values only:

1. **Operating system:** compare the Windows edition and version with `Settings > System > About`.
2. **Processor:** compare the CPU model and logical processor count with `Task Manager > Performance > CPU`.
3. **Memory:** compare installed RAM with `Settings > System > About` or `Task Manager > Performance > Memory`. Windows may reserve a portion for hardware, so usable memory can be lower than installed memory.
4. **Volumes:** compare drive letters and approximate capacities with `File Explorer > This PC`. Mounted volumes and physical disk devices are different concepts; do not expect the counts to match exactly.
5. **Manufacturer/model:** compare with `Settings > System > About` or the device label. Some custom-built PCs may expose a motherboard vendor or generic model instead of the retailer's brand.

Pass the first in-app test when the application opens without crashing, the System Facts action returns a snapshot, and these basic values are reasonably consistent with Windows. Missing sensors, GPU vendor telemetry or SMART information do not fail this first test; they are separate capability checks.

## 6. What not to test yet

For the first run, do not:

- Run any item in Repair Center.
- Install Windows, driver or application updates from CrossDiag.
- Run CPU/RAM benchmark or memory smoke tests.
- Run storage self-tests or destructive disk tools.
- Change BIOS/UEFI settings, Secure Boot or boot order.
- Write an ISO to a USB drive or install an operating system.
- Run the application as Administrator just to make a warning disappear.

After System Facts passes, the next safe feature test is a **read-only Storage inventory**. Install `smartmontools` separately only if you intend to test SMART collection; verify `smartctl --version` in a new terminal, restart CrossDiag, then run the read-only storage scan. A missing `smartctl` result should be recorded as unavailable, not as a failed drive.

## 7. What to send back if something fails

Send:

- The exact command you ran.
- The complete error text (copy/paste, not a paraphrase).
- The generated `test-results/first-pc-test-*.txt` report, after reviewing it for PC model details you do not wish to share.
- Whether the failure happened during dependency installation, TypeScript build, Rust compilation, app launch or the System Facts action.

Never send passwords, updater private keys, BitLocker recovery keys, Windows product keys, drive serial numbers or private signing files.
