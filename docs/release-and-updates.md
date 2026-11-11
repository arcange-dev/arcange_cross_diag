# Signed application updates and release process — v1.0.0-rc.2

CrossDiag uses the official Tauri updater plugin. Application updates are **not enabled in local development builds**. A release build enables the updater only after a maintainer supplies a matching updater public key and private signing key.

## How in-app updates work

1. The maintainer creates a Tauri updater key pair once. The public key is embedded in the signed release build; the private key stays secret.
2. A tagged GitHub Actions release builds the Windows and Linux packages and updater artifacts.
3. Tauri signs updater artifacts using `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. `tauri-apps/tauri-action` uploads the installers, signatures and `latest.json` manifest to a **draft GitHub Release**. The action creates the static updater manifest from the platform bundles and signatures it uploaded; keep the Windows and Linux release jobs on the same tag/release.
5. The maintainer reviews and publishes the draft release.
6. Users open Settings, choose **Check now**, review the version and notes, then explicitly choose **Download & install**. Tauri verifies the update signature before installation. The UI asks for confirmation before downloading/installing; no silent auto-install is performed.

The release endpoint is derived from `GITHUB_REPOSITORY`, so the same workflow works in the actual repository where this project is published. For RC builds, the updater endpoint is pinned to the exact tag because GitHub defines its latest release as the latest published full (non-prerelease, non-draft) release. Stable builds use `/releases/latest/download/latest.json`. The project must be pushed to a public GitHub repository with Releases enabled before production updates can be served.

## One-time signing key setup

Install the Tauri CLI dependencies and generate the key pair on a trusted development machine:

PowerShell:

```powershell
npm run tauri signer generate -- -w "$HOME/.tauri/arcange-crossdiag.key"
```

Linux:

```bash
npm run tauri signer generate -- -w "$HOME/.tauri/arcange-crossdiag.key"
```

The command prints the public key. Keep the private key file and password in a secure password manager or offline backup. **Never commit the private key, add it to a repository variable, paste it into an issue, or include it in the ZIP source package.** Losing the private key prevents publishing updates accepted by already installed copies; plan a key-rotation strategy before release.

In the GitHub repository settings, configure:

- Repository **Variable** `TAURI_UPDATER_PUBLIC_KEY`: the public key content printed by `tauri signer generate`.
- Repository **Secret** `TAURI_SIGNING_PRIVATE_KEY`: the contents of the private key file (Tauri also accepts a path, but secret content is simplest in CI).
- Repository **Secret** `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the key password. Set it to an empty secret only if the key was generated without a password.

The workflow refuses to prepare a release if the public key variable or private signing-key secret is absent, if the public-key variable appears to contain private-key material, or if the Git tag does not match the application version. Before the first public release, complete every item in [the production-readiness checklist](production-readiness-checklist.md), including a real signed upgrade test. The Tauri updater also fails closed if update signatures do not validate.

## Publishing a release

1. Update the version consistently in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Update `RELEASE_NOTES.md` and run:

```bash
npm install
npm run test:source
npm run test:security:source
npm run test:release
npm run test:release-config
npm run test:storage-trends
npm run build
npm run test:rust
```

3. For the first candidate, use the prerelease tag `v1.0.0-rc.2`. The workflow marks `-rc.` tags as prereleases and keeps the release in draft for review. The candidate uses its exact release-tag updater manifest; a later RC install is manual until a newer candidate is installed. Stable `v1.0.0` builds switch to the latest-stable endpoint:

```bash
git tag v1.0.0-rc.2
git push origin v1.0.0-rc.2
```

4. The `Publish signed desktop release` workflow first runs source-security checks, npm and Cargo dependency audits, Rust formatting/tests and the frontend build on each native runner. It then uses the Tauri action to create a draft release and upload native bundles plus signed updater artifacts and `latest.json`.
5. Review the generated installers on real test machines before publishing the draft. Check signatures, checksums, upgrade paths, clean installs and rollback/recovery instructions.
6. Publish the GitHub draft release. The in-app updater endpoint becomes available once `latest.json` is downloadable from the latest published release.

Do not publish the v1.0.0-rc.2 candidate as a stable release until the target GitHub repository exists and signing secrets/variables are configured. The source ZIP itself is not a signed desktop installer.

## Platform packaging

The release workflow builds on native operating systems because native Tauri bundling/signing and dependencies are platform-specific. Current automation targets x86_64 Windows and x86_64 Ubuntu runners; ARM64 builds are not part of this release matrix:

- Windows runner: NSIS and MSI installers plus the updater artifacts selected by the Tauri bundler.
- Ubuntu runner: AppImage, DEB and RPM packages where the host has the required bundling tools, plus AppImage updater artifacts.

Linux auto-update currently targets the AppImage updater format. Users installed through a distribution package manager (DEB/RPM) should update through the new package release and their normal package workflow unless an explicitly supported updater format is configured and tested.

Windows Authenticode signing is separate from Tauri updater signing. The release pipeline does not claim Windows publisher identity or SmartScreen reputation. Configure Authenticode signing in a separate protected workflow with a trusted code-signing certificate before broad distribution.

## Trust model

- HTTPS is required for production updater endpoints.
- Updates must verify against the embedded public key; unsigned updates are rejected.
- The private signing key is only injected into the release workflow from GitHub Secrets.
- Update checks are user initiated in Settings; no background download or silent installation occurs.
- Tauri's updater signature authenticates the update bundle, but users should still install only releases published by the project's trusted repository.
