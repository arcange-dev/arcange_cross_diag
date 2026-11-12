# Arcange CrossDiag — Base44 handoff

## What is included

- `desktop-source/arcange-crossdiag-v1.0.0-rc.2/`: the complete extracted 82-file Rust/Tauri + React/TypeScript desktop source tree.
- `arcange-crossdiag-v1.0.0-rc.2-original.zip`: the original release archive, unchanged.
- `BASE44_MASTER_PROMPT.md`: paste this into Base44 to build the web companion.
- `BASE44_ARCHITECTURE_AND_MIGRATION.md`: explains what Base44 should own and what must remain local.
- `BASE44_DATA_MODEL.md`: proposed Base44 entities, fields, relationships and permissions.
- `SOURCE_MANIFEST.txt`: inventory and SHA-256 of the original archive.

## Important architecture decision

The supplied application is a **native desktop application** built with Tauri 2, Rust, React and TypeScript. Base44 is suitable for building a web dashboard/backend companion, but the Base44-hosted browser app cannot directly read a user's Windows Event Log, Linux journal, SMART devices, GPU sensors, BIOS or physical ports. Those checks must continue to run in a trusted local Rust/Tauri agent.

Recommended design: keep `desktop-source/` as the local diagnostics agent and use Base44 for authenticated multi-device management, telemetry history, cases, alerts, reports and technician collaboration. The local agent sends explicitly approved, privacy-minimized diagnostic snapshots to Base44 over HTTPS.

Do not ask Base44 to replace Rust hardware access with fake browser data or server-side shell commands.

## Suggested Base44 workflow

1. Create a new Base44 app named **Arcange CrossDiag Cloud** (or open your existing Base44 workspace).
2. Paste the full contents of `BASE44_MASTER_PROMPT.md` into the Base44 builder.
3. Ask Base44 to implement in phases: design shell and authentication first; entities and permissions second; dashboards and device records third; ingestion endpoints and tests fourth.
4. Review `BASE44_DATA_MODEL.md` and ensure row-level permissions are implemented before adding real device records.
5. Keep the local Rust agent as the source of hardware measurements. Do not expose an unauthenticated localhost server or accept arbitrary remote commands.
6. Connect the new Base44 app to GitHub using Base44's GitHub integration if your plan includes it. Use a new dedicated repository or branch; do not overwrite the current desktop source repository.

## Source status

This package contains the complete source files from release `v1.0.0-rc.2`, not a compiled installer. The existing project README says it is a release candidate and requires native Windows/Linux build and hardware validation before stable release. The source archive SHA-256 is recorded in `SOURCE_MANIFEST.txt`.
