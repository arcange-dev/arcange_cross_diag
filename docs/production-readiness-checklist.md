# Production-readiness checklist — v1.0.0-rc.2

This checklist is intentionally not marked complete. A source ZIP and green static checks are not the same thing as a verified desktop release.

## Repository and build reproducibility

- [ ] Push the source to the intended public GitHub repository.
- [ ] Run `npm install` on a network-connected machine and commit `package-lock.json`.
- [ ] Confirm the Node.js and npm versions in `package.json` match the supported developer environment.
- [ ] Run `npm run test:source`, `npm run test:security:source`, `npm run typecheck`, `npm run build`, `npm run test:rust` and `cargo fmt --all -- --check`.
- [ ] Obtain green Windows and Ubuntu GitHub Actions builds.
- [ ] Review npm and Cargo audit findings; do not blindly suppress advisories.

## Signing and update channel

- [ ] Generate one Tauri updater key pair on a trusted machine.
- [ ] Add the updater public key as the repository variable `TAURI_UPDATER_PUBLIC_KEY`.
- [ ] Add the updater private key and password as repository secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- [ ] Confirm no private key is present in Git history, release archives, issue attachments or build logs.
- [ ] Run a signed draft release and verify the published `latest.json` manifest and signatures.
- [ ] Install an older build, update it through the in-app updater, and verify the application restarts and retains local case/history data.
- [ ] Test an invalid signature and confirm the updater rejects it.
- [ ] Document key backup and rotation before the first public production release.

## Platform installation checks

- [ ] Test a clean Windows install on a supported Windows 10/11 machine.
- [ ] Test an upgrade over the previous Windows build and an uninstall.
- [ ] Test AppImage installation and in-app update on a supported Ubuntu machine.
- [ ] Test DEB/RPM packages through their package-manager workflows; do not assume AppImage updater support applies to DEB/RPM.
- [ ] Confirm the application runs without administrator/root privileges for ordinary read-only inventory.
- [ ] Confirm specific elevated operations explain why elevation is needed and fail safely when declined.
- [ ] Check SmartScreen/OS publisher warnings. Tauri updater signing is not Windows Authenticode signing.

## Diagnostic quality and safety

- [ ] Test drive-health parsing against representative SATA, NVMe and unsupported drive outputs.
- [ ] Confirm unknown drive endurance stays unknown and critical warnings override the user threshold.
- [ ] Test systems with missing temperature, fan and battery sensors.
- [ ] Test CPU and RAM smoke checks with low available memory and cancellation.
- [ ] Test package manager discovery on each declared Linux distribution family.
- [ ] Verify selected package updates are re-discovered immediately before installation.
- [ ] Verify repair commands are allow-listed and arbitrary frontend command strings are rejected.
- [ ] Verify recovery image hashing and removable-media inventory never write to a disk.
- [ ] Inspect JSON/PDF reports for sensitive identifiers before sharing.

## Public release decision

- [ ] Review draft release notes and all generated installers.
- [ ] Publish only after all platform, security and updater checks are reviewed by a maintainer.


## v0.8.0 runtime readiness and bundle validation

- [ ] Runtime Readiness reports only PATH presence and does not execute optional tools.
- [ ] Check missing-tool install hints on a Windows test machine.
- [ ] Check missing-tool install hints on supported Linux distributions.
- [ ] Build native bundles on Windows and Ubuntu CI runners.
- [ ] Confirm `npm run test:bundle` detects an installer on each runner.
- [ ] Review bundle signatures/checksums and clean-install behavior before publishing.


## v0.9.0 storage trend validation

- [x] Add SMART history comparison and local CSV export.
- [x] Preserve unknown metrics rather than treating them as zero.
- [x] Distinguish normal cumulative power-on-hour increases from worsening health indicators.
- [x] Protect CSV text cells from spreadsheet formula injection.
- [x] Add deterministic Node.js regression tests and run them on Windows/Linux CI.
- [ ] Complete frontend type-check/build with installed dependencies.
- [ ] Complete native Rust test/build on Windows and Linux runners.
- [ ] Verify a real hardware SMART history comparison on multiple SSD/HDD models.


## v1.0.0-rc.2 release candidate gates

- [ ] Confirm `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` all say `1.0.0-rc.2`.
- [ ] Run `npm run test:release-config` and confirm the generated updater configuration includes only the public key and an HTTPS endpoint.
- [ ] Confirm the RC updater manifest uses its exact release tag (GitHub latest excludes prereleases) and the stable build uses the latest stable manifest.
- [ ] Configure the repository public-key variable and private-key secrets; verify the release preflight fails when either is missing.
- [ ] Run `npm run test:ci` on a network-connected development machine after committing `package-lock.json`.
- [ ] Confirm the tagged `v1.0.0-rc.2` workflow creates a GitHub draft prerelease, not a stable published release.
- [ ] Complete the platform and signed-upgrade checks above before promoting to `v1.0.0`.
