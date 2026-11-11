# Arcange CrossDiag v1.0.0-rc.2

This release candidate adds a guided first-PC acceptance test for Windows, strengthens the initial developer setup path, and documents the first safe in-app smoke test.

## Changes

- Added `scripts/first-pc-test.ps1`, a non-elevated, read-only Windows environment and system-inventory preflight.
- Added `npm run test:first-pc:windows` to run that preflight and save a local text report.
- Added a step-by-step first-PC test guide covering setup, application launch, read-only inventory checks, comparison with Windows Settings/Task Manager, and safe follow-up SMART testing.
- Added regression coverage for the first-PC test script contract.
- Synchronized package, Tauri, Cargo, release preflight and updater test versions to `1.0.0-rc.2`.

## Safety

The first-PC script does not run SMART scans, install packages, change system settings, modify drivers, write to disks, or request elevation. It records only general system facts and tool availability; it deliberately excludes serial numbers, MAC addresses, usernames and recovery information.

## Validation limitation

Source-level Node.js audits and tests can run in the packaging environment. A complete dependency-aware frontend build and native Rust/Tauri build still require npm dependencies, Rust/Cargo, and the platform-specific Tauri prerequisites on a Windows or Linux development machine.
