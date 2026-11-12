# CrossDiag platform test matrix — v1.0.0-rc.2

This is the acceptance checklist for real machines and CI runners. A source-level audit is not a substitute for running the application on actual hardware.

## Automated checks

| Check | Windows runner | Ubuntu runner | Pass condition |
|---|---:|---:|---|
| Source contract audit | Yes | Yes | `npm run test:source` exits 0 |
| Security source audit | Yes | Yes | `npm run test:security:source` exits 0 |
| Version/tag preflight | Yes | Yes | package, Tauri, Cargo and tag versions match |
| npm dependency audit | Yes | Yes | No high-severity advisories accepted by policy |
| TypeScript + Vite build | Yes | Yes | `npm run build` exits 0 |
| Rust formatting | Yes | Yes | `cargo fmt --check` exits 0 |
| Rust unit tests | Yes | Yes | `cargo test` exits 0 |
| Cargo dependency audit | Yes | Yes | `cargo audit` passes the configured policy |
| Native desktop packaging | Yes | Yes | Tauri bundle command exits 0 |
| Bundle output validation | Yes | Yes | At least one `.exe`/`.msi` or `.AppImage`/`.deb`/`.rpm` exists as appropriate |

## Manual Windows acceptance

- [ ] Install on a clean supported Windows 10/11 x64 test machine.
- [ ] Launch as a standard user; the app must not demand administrator privileges for ordinary inventory.
- [ ] Check system inventory and compare reported CPU/RAM/volumes with Windows Settings/Task Manager.
- [ ] Open Settings → Runtime readiness; verify `smartctl`, `powershell`, `winget` and GPU tools are marked according to the actual PATH.
- [ ] Install smartmontools from its official source, restart CrossDiag and check that the tool is discovered.
- [ ] Run a SMART scan on at least one SATA/NVMe drive and compare raw attributes with `smartctl -a` output.
- [ ] Run hardware inventory and compare device/problem-device records with Device Manager.
- [ ] Check update discovery without installing anything; review a selected update before confirming installation.
- [ ] Test the Windows Update Settings handoff.
- [ ] Export a JSON report and use the print dialog to save a PDF.
- [ ] If using updater testing, validate a signed update from a non-production test release before enabling general distribution.

## Manual Linux acceptance

- [ ] Install on a clean supported Ubuntu 24.04 x86_64 test machine; separately test Fedora/Arch/openSUSE adapters where claimed.
- [ ] Launch as a standard user; verify only intended operations request PolicyKit/root authorization.
- [ ] Compare CPU/RAM/volume inventory with standard OS tools.
- [ ] Open Settings → Runtime readiness and compare tool detection with `command -v` results.
- [ ] Compare SMART data with `smartctl -a` for a supported drive.
- [ ] Compare PCI/driver information with `lspci -nnk` where pciutils is installed.
- [ ] Compare sensor readings with `sensors`; verify missing sensors remain unknown.
- [ ] Inspect fault evidence against `journalctl -b -p err` while respecting current-user log permissions.
- [ ] Review package update discovery before authorizing a package installation or distribution upgrade.
- [ ] Confirm that removable-media inventory does not write, format or mount a device.
- [ ] Export a JSON report and save the print view to PDF.

## Hardware diversity matrix

Before calling the release production-ready, record actual results on at least:

- [ ] HP or Dell Intel laptop (integrated GPU, NVMe, battery).
- [ ] Lenovo or ASUS AMD laptop (integrated Radeon GPU, NVMe, battery).
- [ ] Desktop with NVIDIA discrete GPU.
- [ ] Desktop with AMD discrete GPU.
- [ ] Intel desktop with integrated graphics only.
- [ ] PC with SATA HDD and SATA SSD.
- [ ] PC with NVMe SSD behind any storage/controller mode used by the test fleet.
- [ ] A system without supported temperature/fan sensors to verify the unknown/unavailable path.
- [ ] A system without a battery to verify desktop behavior.

For each test, retain OS version, model family, CrossDiag build, tool versions, permission context, expected result, actual result and limitations. Remove hostnames, serials, MAC addresses and private event text before publishing test logs.

## Exit criteria

A platform is not production-ready merely because CI builds it. Require successful clean install, launch, core inventory, representative SMART scan, optional-tool readiness checks, report export, update/recovery safeguards, and a signed updater upgrade test on real Windows and Linux machines before publishing a stable release.


## v0.10.0 storage trend regression tests

The platform CI workflow runs `npm run test:storage-trends` on both Windows and Linux before the frontend/native build. These tests use fixed sample SMART scan objects and do not require physical drives or invoke `smartctl`.
