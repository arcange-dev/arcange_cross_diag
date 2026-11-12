# v1.0.0-rc.1 validation report

## Environment

- Node.js 22.16.0 and npm 10.9.2 are present.
- Rust compiler, Cargo and rustfmt are absent.
- Optional runtime tools `smartctl`, NVIDIA-SMI, AMD SMI, Intel GPU Top and `lspci` are absent in this packaging environment; `lsblk` is available.
- `node_modules` and `package-lock.json` are absent. An npm lockfile-generation/install attempt timed out while contacting the npm registry.

## Passed checks

- `npm run test:source` — version synchronization, all 22 frontend-to-Rust Tauri commands and platform/release contracts.
- `npm run test:security:source` — 55 text files scanned; no PEM private-key block found; capability, updater, secret-handling and WebView security rules passed.
- `npm run test:release` — `v1.0.0-rc.1` tag/version consistency across package, Tauri and Cargo manifests.
- `npm run test:release-config` — 11/11 Node.js tests passed. Coverage includes generated public-key-only updater config, missing/misplaced key rejection, signing secret checks, exact tag matching, prerelease tag pinning and stable endpoint selection.
- `npm run test:storage-trends` — 8/8 Node.js tests passed.
- TypeScript/TSX syntax-only transpilation — 7/7 implementation files passed. Declaration files were excluded because `transpileModule` is not intended to emit `.d.ts` files.
- `node --check` — all files under `scripts/*.mjs` and `tests/*.mjs` passed syntax checks.
- JSON, Cargo TOML and GitHub Actions YAML parsing passed.
- Temporary updater configuration was removed after test execution.
- ZIP integrity and SHA-256 checksum verified.

## Blocked / not run

- Full `npm run typecheck` — attempted, but blocked by missing React, Tauri and Vite dependencies/type declarations; the command failed with unresolved-module errors and consequent JSX/type errors. This does not establish whether additional TypeScript errors exist once dependencies are installed.
- `npm run build` — not completed because frontend dependencies are unavailable.
- `cargo fmt`, `cargo test`, Rust compilation and Tauri bundling — not run because Rust/Cargo/rustfmt are missing.
- Real Windows/Linux installers, hardware diagnostics on physical PCs, signed updater download/install and in-place data-retention upgrade — not tested.
- `package-lock.json` could not be generated because npm registry access timed out. It remains a reproducibility blocker before stable release.

## Stable v1.0.0 release gates

1. Run `npm install` on a connected development machine and commit `package-lock.json`.
2. Install Rust 1.90+ and the platform-specific Tauri prerequisites.
3. Run `npm run test:ci`; resolve all type-check, build, formatter, Rust test and bundle-validation failures.
4. Obtain green Windows and Ubuntu CI results and retain the actual installer artifacts.
5. Configure the updater public key as a repository variable and the signing private key/password as repository secrets.
6. Publish the RC draft as a prerelease, test a signed update and confirm the RC app uses its tag-pinned manifest.
7. Test clean installs and upgrades on real Windows and Linux PCs; verify local SQLite case/history data persists.
8. Promote to stable `v1.0.0` only after maintainer review of platform coverage, security audits and diagnostic limitations.

A source archive and source-level tests do not constitute a verified desktop release.
