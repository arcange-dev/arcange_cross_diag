# CrossDiag architecture and diagnostic contract

## Current release candidate: v1.0.0-rc.2

CrossDiag is a local-first Windows/Linux desktop technician tool. It uses a shared React/TypeScript interface, Tauri 2 IPC and a Rust application core. Operating-system-specific collection and execution live behind platform adapters. The v0.6.0 release added per-core CPU readings, optional NVIDIA-SMI, AMD SMI CLI and Intel GPU Top telemetry, vendor support profiles, evidence-based driver findings, a bounded multi-core benchmark, a six-pattern RAM smoke test, recovery image checksum/media inventory workflows, and a native in-process background storage scheduler. The v0.7.0 release added the signed Tauri updater, explicit user approval/install flow, versioned release preflight, native Windows/Linux release automation, tighter WebView security settings, dependency audits, CodeQL and Dependabot. The v0.8.0 release adds the in-app Runtime Readiness panel and post-build installer validation. The v0.9.0 release adds local storage health trend comparisons, direction-aware metric interpretation, and CSV evidence export. The native scheduler continues while the desktop process is running and stops when the application exits.

## Implemented Tauri command groups

1. `get_system_snapshot`: OS/kernel/host/architecture, manufacturer/model where exposed, CPU/core and memory inventory, mounted volumes.
2. `scan_storage_health` / `get_storage_history`: fixed `smartctl` enumeration and JSON SMART collection, normalized evidence, configurable endurance threshold and transactional local SQLite history.
3. `get_hardware_diagnostics`: CPU usage/frequency, memory, GPU/DIMM, driver/device inventory, USB/display, network, firmware/TPM/Secure Boot, battery and supported sensor readings.
4. `run_performance_test`: bounded single-core and multi-core CPU calculation benchmarks plus a user-space six-pattern RAM read-back smoke test. GPU telemetry is read-only through NVIDIA-SMI, AMD SMI CLI and Intel GPU Top on supported OS/driver combinations. These are not full CPU/GPU/RAM stability tests.
5. `check_update_status`: read-only Windows Update Agent/WinGet discovery or detected Linux package-manager, Flatpak and Snap discovery.
6. `install_selected_update`: validates package identifier, re-runs discovery and verifies an exact source/identifier match before launching the package manager.
7. `install_operating_system_updates`: Linux-only distribution package upgrade after explicit UI confirmation and PolicyKit/root authorization; Windows uses the official Update Settings handoff.
8. `get_repair_catalog` / `run_repair_action`: fixed, allow-listed action IDs and fixed executable/argument pairs.
9. `open_driver_manager`: Windows Device Manager handoff or Linux distribution guidance.
10. `open_system_update_settings`: Windows Settings handoff or Linux package-manager guidance.
11. `analyze_system_faults`: bounded Windows event-log or Linux current-boot journal evidence.
12. `list_technician_cases`, `create_technician_case`, `update_technician_case`, `delete_technician_case`: SQLite local case records.
13. `get_scan_schedule` / `save_scan_schedule`: SQLite interval and threshold preference for a native background thread that runs periodic read-only SMART scans while the application process remains open.
14. Vendor support profiles: matched manufacturer/model family, official support URL, diagnostic utility hints and model-dependent firmware/boot-menu hints. Unknown vendors are not assigned guessed official support URLs.
15. Driver findings: reports Windows Configuration Manager problem codes and selected plain-language explanations, unsigned Windows driver records, missing driver-version fields, and Linux PCI bound-driver/candidate-module distinctions. It does not infer driver age or compatibility from version strings.
16. `get_runtime_readiness`: read-only checks for optional tools on PATH, with platform-specific installation hints and no process execution.
17. Signed app updates: Tauri updater plugin checks the configured HTTPS manifest, verifies artifact signatures against an embedded public key, downloads only after an explicit user action, and restarts through the process plugin. Local development builds disable this workflow.

## Platform adapters

### Windows

- System identity and hardware: CIM/WMI, PnP, signed drivers, network adapter statistics, battery, BIOS, Secure Boot and TPM cmdlets.
- Storage: `smartctl` JSON output.
- Updates: Windows Update Agent search + WinGet listing. Selected WinGet package updates may be initiated after a second native discovery check. Windows OS updates are handed off to Settings.
- Repair: `ipconfig /flushdns`, SFC `/verifyonly`, DISM `/CheckHealth` and automatic service review. SFC/DISM checks require CrossDiag to be started as Administrator; the operations are verification-only.
- Drivers: installed driver inventory and Device Manager handoff; driver installation and rollback are not performed.
- Logs: bounded critical/error events from the System log over the last seven days.

### Linux

- Hardware: `/sys`, `/proc`, DMI, EFI variables, sysfs `hwmon`, DRM, network, USB, EDAC, power supply and optional `lspci`.
- Storage: `smartctl` JSON output.
- Updates: detected package manager based on recognized `/etc/os-release` identity/family (`apt-get`, `dnf`, `pacman`, `zypper`) plus optional Flatpak/Snap. Unknown distribution families are marked unsupported rather than guessing a package manager. Selected package updates are rechecked against the latest discovered package/source. Distribution-wide upgrades are a separate confirmed action.
- Elevation: `pkexec` for system package-manager actions when not root; Flatpak remains in the invoking user's context so per-user installations are not silently switched to root.
- Repair: DNS cache flush via `resolvectl`, failed systemd-unit listing and package database inspection (`dpkg --audit` or `rpm -Va`) when present.
- Drivers: PCI/kernel-driver inventory where `lspci` is installed; use the distribution's official utility or manufacturer source for changes.
- Logs: bounded current-boot `journalctl` error-priority records.

## Shared diagnostic result contract

Every diagnostic model should retain component/operation, measurement and unit, source, status/severity, timestamp, limitations and recommended next action. Unsupported and unknown values must remain explicit. Never substitute a guessed percentage or zero reading for unavailable data.

## Persistence

- SMART scan history: local SQLite `storage_scans` table under the Tauri application-data folder, last 100 scans. Earlier `storage-history.json` data is migrated once and retained as a backup.
- Technician case records: `crossdiag.sqlite3`, table `technician_cases`.
- Schedule preferences: same SQLite database, `app_settings` table.
- JSON evidence reports are downloadable. The formatted PDF view uses the system print dialog; CrossDiag does not silently write a PDF file.

## Scheduled scan behavior

The Tauri setup hook starts a native worker thread. It checks the SQLite schedule every 30 seconds, reads the last scan timestamp and starts a read-only SMART scan when the selected interval has elapsed. The configured endurance threshold is saved with the schedule. The worker runs only while CrossDiag is open; it is not a Windows service, systemd user timer or login-start task and stops when the application exits. The React timer only refreshes the displayed local scan history; it does not trigger scans.

## Signed app updater contract

- Local browser/development builds set no updater enable flag and do not invoke the updater.
- The signed release configuration is generated by `scripts/prepare-release.mjs` from the GitHub repository public key variable and repository slug. The generated file is gitignored. `-rc.` builds use a tag-pinned GitHub release asset endpoint; stable builds use the latest stable release endpoint because GitHub’s latest-release route excludes prereleases.
- The private signing key is injected only by the protected GitHub Release workflow using GitHub Secrets. It is never written into the project tree or source ZIP.
- The Settings screen requires the user to check for updates, review version/release notes and confirm installation. Signature validation is enforced by the official Tauri updater plugin.
- Linux in-app updates use the AppImage updater format. DEB/RPM users should use the corresponding package update/reinstall until a verified package-manager updater flow is provided.
- Windows updater signing does not equal Authenticode signing. Windows publisher identity must be configured separately.

## Process execution and security contract

- Frontend calls named Tauri commands; no command accepts free-form shell text.
- Update IDs use a strict allow-list character policy, supported sources are explicit, and the native layer rechecks that the target is present in the latest update discovery result.
- Repair actions are fixed IDs mapped to fixed programs and arguments.
- External process calls have explicit timeouts and an 8 MiB stdout/stderr limit.
- Linux system upgrades use the normal package manager and require `pkexec` or root authorization.
- No firmware flashing, BIOS setting changes, destructive disk action, bootloader edit or unattended OS installation is included.

## Known limitations

Live GPU telemetry uses NVIDIA-SMI and, on supported Linux systems, AMD SMI CLI and Intel GPU Top. Provider and metric availability depend on GPU family, driver, tool version and permissions. Sensor availability varies by hardware/driver/firmware; Windows ACPI thermal-zone data may not be CPU/GPU temperature; Windows `Win32_Fan.DesiredSpeed` is a target and not actual RPM. Battery health requires unambiguous capacity counters. Physical port and cable testing requires equipment. Event logs are evidence and not root cause. Driver installation/rollback and bootable USB writing remain guided/manual workflows.


## v0.6.0 platform-specific collectors

- GPU providers are independent adapters: NVIDIA-SMI, AMD SMI CLI on supported Linux systems, and Intel GPU Top on supported Linux systems. Their provider/source strings are part of the telemetry result. Missing provider fields remain null.
- Windows problem-device records include Configuration Manager codes from `Win32_PnPEntity` where WMI exposes them; selected codes are converted to plain-language findings.
- Linux `lspci -nnk` output distinguishes a bound driver from listed candidate kernel modules. Candidate modules alone are not treated as a failure.
- The recovery-image verifier streams a local image into SHA-256 and optionally compares it with a supplied expected digest. Removable-media discovery is inventory-only and never writes to a disk.
- The RAM smoke test performs six read-back patterns on a bounded application-owned allocation. It is not a substitute for a boot-time memory diagnostic.


The v0.9.0 release adds a frontend storage-history comparison and local CSV evidence export. Trend rules explicitly preserve unknown readings and do not treat cumulative power-on hours as a fault.


## v0.10.0 persistence and process-output hardening

Storage scan history is stored in the existing local SQLite database. Inserts and retention cleanup happen in one transaction; first-run migration from the earlier JSON file is serialized with an immediate transaction and only marked complete after successful import. The original JSON file is retained as a local backup. External process output is bounded to 8 MiB and decoded with replacement characters rather than failing on non-UTF-8 output.


## Native process output

The Rust command adapter uses fixed executable names and validated argument vectors rather than shell interpolation. It applies a per-command timeout and an 8 MiB output limit. Output bytes are decoded with lossy UTF-8 so device names or OEM-encoded Windows text do not abort a diagnostic. Windows PowerShell JSON collectors explicitly set UTF-8 console output before serializing structured records.
