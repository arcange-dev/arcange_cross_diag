# Arcange CrossDiag

**Open-source Windows and Linux PC diagnostics and technician workflows — v1.0.0-rc.2**  
Created by Mukamyi Izere Arcange.

Arcange CrossDiag is a local-first desktop technician application built with **Rust, Tauri 2, React and TypeScript**. It collects hardware and operating-system evidence, explains what the operating system and supported tools report, and provides carefully scoped maintenance and recovery workflows. It is the cross-platform successor direction to [Arcange Windows Technician Toolkit](https://github.com/arcange9/arcange-windows-technician-toolkit).

> CrossDiag is a diagnostic assistant, not a guarantee that hardware is fault-free. Device enumeration does not prove a component is physically healthy. SMART, sensor and event-log results have limitations; physical connector, cable, motherboard, PSU and intermittent faults can require test equipment or inspection.

## v1.0.0-rc.2 — first-PC validation candidate

This is a **release candidate**, not a production-verified stable release. It consolidates the v0.10.0 feature set, strengthens the release gate, and adds a guided first-PC validation workflow:

- `npm run test:first-pc:windows` runs a non-elevated PowerShell preflight and writes a timestamped report under `test-results/`. It checks source files, synchronized versions, Node/npm/Rust/Cargo availability, Windows version, PC model, CPU, RAM, mounted volumes, optional diagnostic tools and current privilege level.
- The preflight is read-only: it does not run SMART scans, stress tests, repairs, updates, driver changes, firmware operations or disk writes. It deliberately omits serial numbers, MAC addresses, usernames and credentials.
- `npm run test:first-pc:script` runs portable Node.js tests to verify the preflight's safety contract on CI.
- [First Windows PC test guide](docs/first-pc-test.md) explains the first application smoke test: launch the desktop app, collect system facts, and compare the reported OS/CPU/RAM/volumes with Windows Settings, Task Manager and File Explorer before attempting any hardware test.

- Package, Tauri and Cargo versions are synchronized to `1.0.0-rc.2`.
- Release preflight accepts SemVer prereleases and requires the exact `v1.0.0-rc.2` tag.
- Tagged `-rc.` builds are marked as GitHub prereleases; the workflow keeps the release as a draft for maintainer review. Their updater endpoint is pinned to the exact candidate tag because GitHub’s latest-release endpoint excludes prereleases; stable builds use the latest stable manifest.
- A signing preflight checks that the updater public-key variable and private-key secret are present without printing either value.
- Automated tests exercise the updater configuration generator, including missing public keys, private-key material mistakenly supplied as public data, exact tag matching, signing-secret checks and stable/RC updater endpoint selection.
- The unified CI test command explicitly includes SQLite migration and bounded command-output regression tests.

Do not publish this candidate as the stable `v1.0.0` release until a dependency lockfile is committed, native Windows and Linux builds pass, and signed installation/upgrade tests succeed on real machines.

## Previous release foundation — v0.10.0

### Desktop and inventory

- In-app Runtime Readiness panel in Settings checks whether optional platform tools are discoverable on PATH and shows install guidance without running or installing them. See [runtime readiness documentation](docs/runtime-readiness.md).

- Tauri 2 desktop application with React + TypeScript interface and shared Windows/Linux architecture.
- Signed Tauri application updater in release builds: user-initiated update checks, release-note preview, download progress, signature verification, and explicit install/restart confirmation. Local development builds keep the updater disabled until a public key and signing endpoint are generated for a release.
- OS, kernel, hostname, architecture, manufacturer/model where reported, CPU/core, memory and mounted-volume inventory.
- Hardware evidence for CPU usage/frequency and per-logical-processor activity, RAM modules where exposed, GPU inventory, drivers, problem devices, USB/display enumeration, network adapters, firmware/TPM/Secure Boot, battery capacity counters, temperature and fan-related readings.
- Read-only vendor telemetry: NVIDIA-SMI, AMD SMI CLI on supported Linux systems, and Intel GPU Top on supported Linux systems. Only fields returned by the provider are displayed; Intel engine activity is labelled as engine activity rather than a standardized whole-GPU score.
- Manufacturer-aware support profiles and official support links for common worldwide brands, including model-family hints, diagnostic utility suggestions and cautious BIOS/boot-menu entry guidance.
- Evidence-based driver findings for OS-reported device problems, selected Windows Configuration Manager codes, unsigned Windows driver records, and Linux PCI driver/module binding. Driver version age is not treated as proof of incompatibility.

### Storage health

- Physical drive enumeration and SMART/NVMe evidence collection through independently installed `smartmontools`.
- Supported SMART status, endurance, NVMe critical warnings, temperature, available spare, selected media-error indicators, power-on hours and selected ATA attributes.
- Configurable remaining-endurance threshold, default **90%**: above threshold is healthy under this user policy; at or below threshold is attention/Bad condition. Critical warnings override this policy.
- NVMe Percentage Used is treated as endurance consumed, not remaining. Missing/unsupported endurance stays unknown.
- Read-only scans; no self-tests, disk writes or firmware modifications are initiated.
- Local SQLite storage-scan history capped at 100 scans, with one-time migration from legacy JSON history.

### Update center

- Read-only update discovery using Windows Update Agent and WinGet on Windows; APT, DNF, Pacman or Zypper on recognized Linux distribution families from `/etc/os-release`, with optional Flatpak/Snap discovery when a supported base package manager is identified. Unknown distribution families are not guessed.
- Selective installation of an individual discovered WinGet or Linux package update after a separate confirmation. Before executing, CrossDiag re-runs update discovery and verifies that the selected identifier/source is still in the latest update list.
- Distribution-wide Linux package upgrade after a prominent confirmation and PolicyKit authorization (`pkexec`) when not already running as root. Supported commands are selected from `/etc/os-release` and the detected distribution family.
- Windows operating-system updates are handed off to the official Windows Update Settings page. CrossDiag does not bypass the Windows Update orchestrator.
- Installer output, exit code and limitations are shown after the operation. No automatic reboot is performed.
- Package-manager metadata can be stale. Review distribution/vendor notices and back up data before significant updates.

### Repair and maintenance center

Fixed, allow-listed actions only; the UI asks for confirmation and the Rust backend rejects unknown action IDs. Current actions include:

- Windows: flush DNS cache; SFC verify-only; DISM CheckHealth; review stopped automatic services.
- Linux: flush `systemd-resolved` DNS cache where available; list failed systemd units; inspect the package database with `dpkg --audit` or `rpm -Va` where available.

The current catalog does **not** run broad automatic repairs, reset the network stack, restart services, delete files, change boot settings, repair files with SFC/DISM RestoreHealth, or modify drivers. Administrator access is required for the Windows SFC/DISM verification actions.

### BIOS, drivers and recovery guidance

- Model/brand-informed firmware setup and one-time boot-menu key hints.
- Official Windows/Linux image links and links to Rufus, Ventoy and balenaEtcher.
- Step-by-step USB preparation, backup, checksum-verification and installation guidance.
- Windows Device Manager handoff and official manufacturer driver support guidance.
- No firmware flashing, BIOS setting writes, bootloader edits, partition changes, or destructive USB writing is automated. Boot-key hints must be checked against the exact model's official manual.

### Technician cases, scheduling and reports

- SQLite database in the Tauri application-data directory for local technician cases, scan-schedule preferences and storage scan history.
- Create, edit status/notes and delete cases. Do not store passwords, BitLocker recovery keys, credentials or other secrets in case notes. The SQLite database is local but is not encrypted by CrossDiag.
- Optional periodic read-only SMART scans every 1, 6, 12, 24, 48 or 168 hours. **Scheduled scans run only while CrossDiag is open**; this is not a background service and does not run while the computer or application is closed.
- JSON evidence export and a formatted print report that can be saved as PDF through the operating system's print dialog.
- Older releases used `storage-history.json`. v0.10.0 migrates that file once into the SQLite `storage_scans` table and retains the original JSON file as a backup.

### Fault evidence

- Bounded Windows System critical/error events from the last seven days.
- Bounded current-boot Linux journal error-priority entries.
- Event text, source, timestamp and identifiers are shown as evidence, not automatic proof of a failed component.

## Current limitations / not implemented

- General-purpose GPU stress suites, standardized cross-machine benchmark scores, full RAM stability testing and built-in Memtest86+ execution. v0.6 provides bounded single-core and multi-core CPU calculation benchmarks plus a 32–512 MiB, six-pattern user-space RAM smoke test; these are not certified stability tests.
- Actual physical testing of USB/HDMI/DisplayPort/audio sockets, cables, fans, PSUs or motherboard signals.
- Automatic driver installation/rollback, vendor firmware utilities, warranty lookup and a complete manufacturer error-code database.
- Direct Windows OS update installation inside CrossDiag; Windows Update is handed off to Settings.
- Flatpak/Snap are individually installable only when the package appears in the latest checked list; system-wide Linux upgrade handles the detected base package manager only.
- BIOS/UEFI setting changes, firmware flashing, bootloader repair, automated partitioning, automated bootable USB writing and unattended OS installation. v0.6 can hash a local ISO/IMG and inventory candidate removable devices, but it does not write to them.
- Scheduled scans while CrossDiag is closed, Windows Task Scheduler/systemd service integration, login-start background execution and automatic PDF file generation without the print dialog. The native scheduler operates only while the desktop process remains running.
- Native Rust compilation and end-to-end desktop runtime have not been verified in this packaging environment; see Validation below.
- Runtime Readiness checks only executable presence on PATH. It does not prove permissions, driver compatibility, sensor support or successful device access.
- Windows Authenticode signing and SmartScreen reputation are not configured. Tauri updater signatures do not replace Windows executable publisher signing.

### Earlier v0.6.0 hardware and recovery additions

- GPU telemetry providers expanded to NVIDIA-SMI, AMD SMI CLI (supported Linux systems) and Intel GPU Top (supported Linux systems). Telemetry source is shown with each reading; unsupported metrics remain unavailable.
- RAM user-space smoke test uses six patterns with configurable 32–512 MiB allocation, subject to a one-quarter available-memory guard. This remains a bounded process-level test, not a boot-time physical-memory certification.
- Windows problem-device collection includes Configuration Manager problem codes where WMI exposes them, with plain-language triage and safe next steps. Linux PCI inventory distinguishes a bound driver from a candidate kernel module.
- Recovery assistant calculates local SHA-256 for ISO/IMG files, optionally compares a publisher-provided expected digest, and inventories removable USB disks. There is deliberately no raw USB writer.
- Cross-platform CI now builds the frontend, runs Rust unit tests and attempts native Windows/Linux desktop bundles, then uploads bundle artifacts for review.



### v0.10.0 reliability changes

- Storage scan history now uses the same local SQLite database as technician cases and settings. Writes are transactional, retention is capped at 100 scans, and concurrent scheduler/manual scans no longer use a read-modify-write JSON file.
- Existing `storage-history.json` data is imported once into SQLite. The legacy JSON file is retained as a backup and malformed files are not overwritten or marked as migrated.
- SQLite uses a five-second busy timeout, WAL journaling and `synchronous=NORMAL` for local concurrent access.
- Native command output is read as bytes under an 8 MiB in-memory cap and decoded lossily, preventing non-UTF-8 device names or Windows console output from aborting a diagnostic. The reader continues draining the child pipe after the cap is reached to avoid child-process pipe backpressure.
- Regression coverage now includes output decoding, missing pipes, output-size limits, SQLite retention ordering, one-time legacy migration and malformed migration preservation.

### Storage trend analysis (v0.10.0)

- Compare the two newest saved SMART scans with per-drive status and metric deltas.
- Identify added/removed device paths and status transitions without claiming device identity from a path alone.
- Missing measurements remain unknown; power-on hours are treated as cumulative usage rather than a failure indicator.
- Export saved SMART evidence, findings and limitations to a local CSV file.
- Run `npm run test:storage-trends` for regression coverage of timestamps, metric deltas, missing data, CSV quoting and warning direction.

## Requirements

- Node.js 22 or newer and npm.
- Rust 1.90 or newer (the Tauri updater plugin requires Rust 1.90).
- Stable Rust toolchain and Cargo via [rustup](https://rustup.rs/).
- Tauri 2 native prerequisites: <https://v2.tauri.app/start/prerequisites/>.
- `smartmontools` on `PATH` for physical-drive SMART scans.
- Linux: optional `pciutils` (`lspci`) improves PCI inventory; `pkexec` and a PolicyKit authentication agent are needed for elevated package operations when CrossDiag is not running as root.

### Install optional SMART tools

Windows PowerShell:

```powershell
winget install --id smartmontools.smartmontools -e
smartctl --version
```

Ubuntu/Debian:

```bash
sudo apt update
sudo apt install smartmontools pciutils intel-gpu-tools
```

Fedora:

```bash
sudo dnf install smartmontools pciutils
```

Arch Linux:

```bash
sudo pacman -S smartmontools pciutils intel-gpu-tools
```

For Intel Linux engine telemetry, `intel_gpu_top` is supplied by `intel-gpu-tools` on supported distributions. For AMD Linux telemetry, install AMD SMI only by following AMD's official ROCm/AMD SMI instructions for the exact supported OS/GPU combination: <https://rocm.docs.amd.com/projects/amdsmi/en/develop/how-to/amdsmi-cli-tool.html>. NVIDIA telemetry requires a compatible NVIDIA driver installation that provides `nvidia-smi`.

Close and reopen CrossDiag after installing a tool so it inherits the updated `PATH`. Do not routinely run CrossDiag as administrator/root; request elevation only for a specific operation when required.

## First test on a Windows PC

Read [docs/first-pc-test.md](docs/first-pc-test.md) before the first real-PC test. After extracting the source and installing prerequisites, run `npm run test:first-pc:windows` from a normal PowerShell window. It only collects read-only environment and system inventory facts and saves a timestamped report under `test-results/`.

## Run locally

```bash
npm run setup
npm run doctor
npm run test:storage-trends
npm run tauri dev
```

Run the new persistence and command-output regression tests, then the full Rust suite and desktop build:

```bash
npm run test:storage-history
npm run test:command-output
npm run test:rust
npm run build:desktop
```

Browser-only interface preview (native diagnostics are unavailable):

```bash
npm run dev
```

Build Windows packages on Windows and Linux packages on Linux unless a separately configured cross-build workflow is used.

## Signed application updates

See [docs/release-and-updates.md](docs/release-and-updates.md) for the one-time signing-key setup, GitHub repository variables/secrets, release workflow, update trust model and platform packaging details. The private updater signing key must never be committed or included in a source archive.

## Architecture

```text
React + TypeScript frontend
   | typed Tauri commands + updater plugin
   v
Tauri 2 desktop shell
   v
Rust application core
   ├── system/hardware inventory adapters
   ├── storage SMART parser and endurance policy
   ├── update discovery + confirmed package operations
   ├── fixed repair-action catalog
   ├── driver/device and recovery guidance
   ├── SQLite technician cases + scan preferences
   ├── JSON history and PDF-via-print report view
   └── Windows APIs / PowerShell / WinGet / Windows Update Settings
       Linux sysfs / hwmon / journalctl / package managers / pkexec
```

The frontend cannot supply an arbitrary executable or shell command. Native operations are exposed as named Tauri commands. Update package identifiers are validated and rechecked against a fresh update discovery result; repair actions use fixed IDs and fixed executable/argument lists. Long-running commands have explicit timeouts and output limits.

## Diagnostic interpretation and safety

- `healthy`, `attention`, `failed` and `unknown` describe available evidence; unknown does not mean healthy or failed.
- SSD endurance is not a guaranteed lifetime prediction. HDDs commonly lack a standardized remaining-life percentage.
- Battery health is distinct from current charge percentage.
- A sensor's source matters: ACPI thermal zones are not necessarily CPU/GPU temperatures; requested fan speed is not actual fan RPM.
- A driver version's age alone is not a fault.
- A port's enumeration or negotiated link speed is not a physical signal-integrity test.
- Updates can cause compatibility issues, require restarts or affect dependent packages. Review the transaction before confirming.
- Firmware updates, BIOS configuration, bootloader, partitioning and OS installation have model-specific and destructive risks and therefore remain guided/manual workflows.
- Back up important data before updates, recovery or repair work.

## Privacy and local data

- No telemetry, cloud account or remote-control service is included.
- Storage history, technician cases and preferences are stored locally in SQLite in the operating-system app-data directory. Older JSON storage history is retained as a migration backup.
- Reports can contain computer model, device paths, MAC addresses, hostnames and event text. Review exports before sharing.
- CrossDiag does not encrypt case notes; never save passwords or recovery keys in them.

## Contributing and security

See [First Windows PC test](docs/first-pc-test.md), [Storage trend analysis](docs/storage-trends.md), [Storage history persistence](docs/storage-history.md), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), [Runtime Readiness](docs/runtime-readiness.md), [Platform test matrix](docs/platform-test-matrix.md), [docs/production-readiness-checklist.md](docs/production-readiness-checklist.md), [docs/release-and-updates.md](docs/release-and-updates.md), [docs/architecture.md](docs/architecture.md), [docs/platform-coverage.md](docs/platform-coverage.md), [docs/vendor-support.md](docs/vendor-support.md), [docs/performance-tests.md](docs/performance-tests.md), [docs/gpu-diagnostics.md](docs/gpu-diagnostics.md), [docs/driver-error-codes.md](docs/driver-error-codes.md) and [docs/safe-operations.md](docs/safe-operations.md).

## Third-party tools and licenses

CrossDiag is MIT-licensed: the license text shipped with the v1.0.0-rc.2 source archive is kept as [`LICENSE-MIT`](LICENSE-MIT). The top-level [`LICENSE`](LICENSE) file holds the Apache-2.0 text from the original repository bootstrap. `smartmontools` is a separate GPL-2.0 project and is not bundled. `pciutils`, WinGet, Flatpak, Snap, package managers and vendor utilities keep their own licenses and terms. Review third-party licensing before bundling external binaries.

- smartmontools: <https://github.com/smartmontools/smartmontools>
- Tauri: <https://v2.tauri.app/>
- React: <https://react.dev/>
- SQLite: <https://sqlite.org/>

## Validation status for this source package

See the detailed [v1.0.0-rc.2 validation report](docs/validation-v1.0.0-rc.2.md), [production-readiness checklist](docs/production-readiness-checklist.md) for checks completed and environment blockers.


In this packaging environment, Node.js 22.16.0 and npm 10.9.2 are available. The source audit, security source audit, RC.2 release preflight, 11 release-configuration tests, 8 storage-trend tests, and 5 first-PC preflight safety-contract tests passed. Six TypeScript/TSX source files passed syntax transpilation with the global TypeScript 5.8.3 compiler; this is not a dependency-aware type-check. JSON, Cargo TOML and GitHub Actions YAML were parsed successfully. The Windows PowerShell preflight is included and wired into the Windows CI runner, but this Linux packaging environment cannot execute that Windows-specific script. Rust/Cargo are not installed here, and npm registry access was unavailable, so native Rust tests, full `tsc` type-check, Vite production build, Tauri desktop build and real Windows/Linux hardware tests remain unverified. A `package-lock.json` could not be generated; run `npm install` on a connected Windows PC and keep the resulting lockfile for reproducible installation. The signed release workflow requires `TAURI_UPDATER_PUBLIC_KEY` as a repository variable and `TAURI_SIGNING_PRIVATE_KEY` plus `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as repository secrets. A green Windows/Linux CI run and successful real-PC acceptance tests are required before stable v1.0.0.

## v1.0.0-rc.2 validation note

The source package also includes Rust tests for transactional SQLite storage-history retention, one-time migration from legacy JSON, preservation of malformed legacy files, non-UTF-8 command output, missing pipes and command-output size limits. These native tests require Rust/Cargo and were not executed in this packaging environment. The first-PC PowerShell test is intentionally read-only and is run on the Windows CI runner or the user’s Windows PC.
