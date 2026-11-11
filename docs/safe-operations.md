# Safe operations reference

CrossDiag separates discovery, selected changes and high-impact/manual workflows.

| Operation | Current behavior | User confirmation | Permission / notes |
|---|---|---|---|
| System/hardware inventory | Read-only collectors | Scan button | Normal permissions where possible |
| SMART collection | Read-only smartctl JSON | Scan button | Some controllers need additional device permissions |
| Storage scheduled check | Read-only SMART | Opt-in schedule | Runs only while CrossDiag remains open |
| WinGet/package update | One selected item, latest scan recheck | Per-package confirmation | WinGet context or Linux package manager authorization |
| Linux distribution upgrade | Standard detected package-manager upgrade | Dedicated warning confirmation | Root or PolicyKit (`pkexec`) |
| Windows OS update | Open Windows Update Settings | User completes there | Windows controls download/install/restart |
| DNS cache flush | Fixed `ipconfig` or `resolvectl` action | Per-action confirmation | Only changes local resolver cache |
| SFC/DISM checks | Verify-only / CheckHealth modes | Per-action confirmation | Run CrossDiag as Administrator |
| Failed service review | Read-only listing | Per-action confirmation | Does not start/restart services |
| Driver maintenance | Inventory + Device Manager/vendor guidance | User-directed | No unsigned driver installation |
| BIOS/UEFI/recovery | Guided instructions and official links | User-directed | No firmware or boot-setting writes |
| Bootable USB preparation | Instructions + external media tools | User-directed | External tool writing can erase the selected USB; verify target carefully |
| PDF report | Print dialog to Save as PDF | User saves file | Includes only data currently collected in the UI |
| Technician cases | Local SQLite CRUD | User actions | Local database is not encrypted; do not store secrets |

## Prohibited implementation patterns

- Do not expose a command runner that accepts arbitrary executable names or shell strings from the frontend.
- Do not use brand name alone to select firmware images or BIOS updates.
- Do not label missing readings as 0, passed, healthy or failed.
- Do not infer drive/CPU/GPU/RAM life percentages without a defined, trustworthy measurement.
- Do not automatically restart, delete, repartition, format, flash, or install an OS.
- Do not run all maintenance actions silently or without a clear summary and confirmation.
