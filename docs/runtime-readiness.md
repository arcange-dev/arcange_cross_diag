# Runtime Readiness panel — v0.8.0

The Settings page exposes a read-only runtime check that tells the user whether selected optional diagnostic utilities are discoverable through the current process `PATH`. It does not execute those utilities, request administrator/root privileges, install packages or change system state.

## Windows tools

| Tool | CrossDiag feature | Notes |
|---|---|---|
| `smartctl` | SMART storage health | Install smartmontools separately. Controller pass-through and permissions vary. |
| `powershell` | CIM, device and event collection | Windows PowerShell is normally present; PowerShell 7 alone may not provide all Windows modules. |
| `winget` | Application update discovery/install | Supplied through Microsoft App Installer; availability depends on Windows build and policy. |
| `nvidia-smi` | NVIDIA GPU telemetry | Requires compatible NVIDIA driver. |
| `amd-smi` | AMD GPU telemetry | Requires a supported AMD SMI runtime and GPU/driver combination. |

## Linux tools

| Tool | CrossDiag feature | Notes |
|---|---|---|
| `smartctl` | SMART storage health | Install smartmontools; device access may require permissions. |
| `lspci` | PCI inventory and kernel binding | Supplied by pciutils. |
| `lsblk` | Removable-media inventory | Supplied by util-linux; inventory only, no writing. |
| `sensors` | Temperatures/fans/voltage | Supplied by lm-sensors; kernel driver support is required. |
| `journalctl` | System fault evidence | Some logs require membership in a privileged group or elevation. |
| `pkexec` | Authorized package-manager operations | Requires PolicyKit configuration and desktop authorization agent. |
| `fwupdmgr` | Firmware metadata discovery | Hardware support varies; CrossDiag does not flash firmware. |
| `apt-get`, `dnf`, `pacman`, `zypper` | Distribution update workflows | Only the package manager matching the detected distribution should be used. |
| `nvidia-smi`, `amd-smi`, `intel_gpu_top` | Vendor GPU telemetry | Each reports a different subset of metrics and requires compatible hardware/drivers. |

## Interpreting results

- **Available** means the executable file was located on `PATH`; it does not prove the program runs or can read the intended hardware.
- **Not found** means CrossDiag could not find the expected executable name on the current `PATH`. Reopen CrossDiag after changing `PATH`.
- A missing tool is not a hardware failure.
- A present tool does not guarantee driver, permission, firmware or hardware support.
- Install tools only from their official project or distribution repositories. CrossDiag intentionally does not install them automatically.
