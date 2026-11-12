# v1.0.0-rc.2 validation report

## Added in this candidate

- Windows read-only first-PC preflight script and guide.
- Portable Node.js tests checking that the PowerShell preflight covers core inventory and does not run diagnostic/repair commands or collect serial/MAC/credential fields.
- Version synchronization across package.json, Cargo.toml and Tauri configuration.
- RC updater endpoint pinned to the exact prerelease tag.

## Checks performed in the packaging environment

- `node scripts/source-audit.mjs` — passed.
- `node scripts/security-source-audit.mjs` — passed.
- `node scripts/release-preflight.mjs` — passed for `v1.0.0-rc.2`.
- `node --test tests/release-config.test.mjs` — 11 tests passed.
- `node --test tests/storage-trends.test.mjs` — 8 tests passed.
- `node --test tests/first-pc-test-script.test.mjs` — 5/5 passed.
- TypeScript/TSX syntax transpilation with TypeScript 5.8.3 — 6 source files passed (syntax only; not a full type-check).
- JSON, Cargo TOML and YAML syntax checks — passed.
- ZIP integrity and SHA-256 verification — passed.
- GitHub Actions YAML, package/Tauri/capability JSON and Cargo TOML parsing — passed.

## Not verified here

A complete `npm install`, dependency-aware TypeScript check, Vite build, Cargo compilation, Rust unit tests, Tauri development launch, native Windows/Linux installers, signed updater installation, and physical PC hardware validation require a connected Windows/Linux build environment with the documented toolchains. The Windows-specific PowerShell preflight is included in the Windows CI workflow and has not been executed in this Linux packaging environment. These are not claimed as passed by this report.

## First PC acceptance test

The required first test is the read-only System Facts smoke test described in [first-pc-test.md](first-pc-test.md). Start as a standard Windows user. Compare OS edition/version with Settings > System > About, processor information with Task Manager > Performance > CPU, memory with Task Manager > Performance > Memory, and mounted volume letters/capacities with File Explorer > This PC. Do not begin with repair, update installation, stress testing, firmware, or USB-writing operations.
