# v0.10.0 validation report

This report distinguishes checks completed in the packaging environment from native checks that require the project toolchain.

## Passed locally

- `npm run test:source` — version consistency, 22 frontend-to-Rust command registrations, core feature contracts and the Windows/Linux CI workflow.
- `npm run test:security:source` — scanned 50 text files; no PEM private-key block found; verified secret ignore rules, Tauri capability scope, updater gating and WebView security settings.
- `npm run test:release` — `v0.10.0` package, Cargo and Tauri versions agree.
- `npm run test:storage-trends` — 8/8 Node.js regression tests passed.
- TypeScript/TSX syntax transpilation — 15/15 source, script and test files passed syntax-only transpilation using the globally installed TypeScript compiler.
- `node --check` — all `.mjs` scripts and tests passed syntax validation.
- JSON, Cargo TOML and GitHub Actions YAML parsing passed.
- The final ZIP archive passed `unzip -t` integrity validation and its SHA-256 checksum was generated.

## Not run / blocked by environment

- `npm run test:storage-history`, `npm run test:command-output`, `npm run test:rust`, `cargo fmt` and native Rust compilation were not run because `rustc`, `cargo` and `rustfmt` are not installed in the packaging environment.
- `npm run build` was attempted but cannot complete because frontend packages and their type declarations (React, Tauri API, Vite and related packages) are not installed. `npm install` cannot retrieve dependencies because the npm registry is not reachable from this environment; an offline lockfile attempt returned `ENOTCACHED`.
- Windows and Linux desktop installers, hardware behavior, signed updater downloads and in-place upgrades have not been tested here.

## Required before calling this production-ready

1. Run `npm install` on a network-connected development PC and commit the generated `package-lock.json`.
2. Install the supported Rust toolchain and Tauri platform prerequisites.
3. Run `npm run test:storage-history`, `npm run test:command-output`, `npm run test:rust`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, `npm run build` and `npm run build:desktop`.
4. Run the Windows/Linux GitHub Actions workflows and review the actual build/test logs and generated artifacts.
5. Test representative PCs, SMART adapters, sensor providers, update sources, recovery workflows and signed upgrades on real hardware.
6. Configure updater signing variables/secrets and complete an end-to-end signed release test before publishing a stable v1.0.0.

Syntax-only transpilation is not a substitute for TypeScript type-checking, Rust compilation or platform runtime testing.
