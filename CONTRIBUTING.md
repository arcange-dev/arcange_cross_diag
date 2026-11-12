# Contributing to Arcange CrossDiag

Thank you for helping improve CrossDiag. Contributions should make diagnostics more accurate, explainable, safe and portable.

## Before opening a pull request

1. Describe the operating system, distribution/version, device model (omit serial numbers) and optional utilities involved.
2. Explain the diagnostic question and the source of the measurement or status.
3. Include a sanitized sample of command/API output when adding a parser. Remove serial numbers, MAC addresses, usernames, hostnames, IP addresses and other identifying information.
4. Explain permissions, false-positive risks and situations where the interface should return `unknown` or `unsupported`.
5. Check the license of every third-party library or external binary. Do not silently bundle GPL or vendor utilities into CrossDiag.

## Development checks

```bash
npm install
npm run doctor
npm run build
npm run test:rust
npm run test:source
npm run test:security:source
npm run test:release
npm run test:ci
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
```

For a platform-specific desktop smoke test:

```bash
npm run tauri dev
```

GitHub Actions runs frontend checks, native compilation and Rust tests on Windows and Linux, plus npm/Cargo dependency audits and CodeQL. Signed release builds require the release signing variables/secrets described in `docs/release-and-updates.md`. Do not use a real signing key in a fork or pull request build.

## Safety requirements

- Keep hardware and operating-system collection read-only unless a separately reviewed action is approved.
- Never build a shell command by concatenating user-provided text.
- Use fixed executables/arguments, bounded timeouts, bounded output and explicit platform adapters.
- Do not invent lifespan percentages, sensor values, device health or fault-code meanings.
- Distinguish measured values, estimates, warnings, unsupported information and confirmed failures.
- Do not silently install updates, change firmware, repartition disks, modify boot configuration or run destructive repairs.
- Add unit tests for parsers and health rules. Test both ordinary and malformed/unavailable input.

## Pull requests

Keep changes focused. Update `README.md`, `docs/platform-coverage.md` and `docs/architecture.md` when behavior or platform support changes. Include test results and list any host-specific behavior that could not be verified.
