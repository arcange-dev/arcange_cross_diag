# Platform coverage matrix — v1.0.0-rc.2

**Implemented** means a collector or workflow exists in source; it does not guarantee that every computer exposes the needed information. **Partial** means OS build, distribution, driver, tool version, permissions or hardware changes the outcome. **Guidance** means the app explains the workflow but does not perform the risky operation.

| Capability | Windows 10/11 | Linux | Important limitation |
|---|---|---|---|
| OS, CPU, memory and mounted volumes | Implemented | Implemented | Virtualization and access restrictions can hide details |
| Runtime Readiness tool presence | Implemented | Implemented | Checks PATH presence only; it does not test execution, permissions or device access |
| Manufacturer/model | Implemented | Partial | DMI/CIM data may be missing |
| GPU inventory | Implemented | Implemented | Inventory is not a functional test |
| NVIDIA live telemetry | Partial through `nvidia-smi` | Partial through `nvidia-smi` | Driver/tool support and permissions apply |
| AMD live telemetry | Not currently collected by CrossDiag | Partial through `amd-smi` | AMD SMI installation, GPU family, driver and JSON fields vary |
| Intel live telemetry | Inventory; no generic live collector | Partial through `intel_gpu_top` | Engine busy counters are not a whole-GPU score; PMU support/access varies |
| RAM module inventory | Partial | Partial via EDAC sysfs where exposed | Many consumer systems hide module details |
| RAM smoke test | Implemented | Implemented | 32–512 MiB process-owned allocation, six patterns; not full physical-memory certification |
| CPU performance smoke tests | Implemented | Implemented | Bounded calculation loops; not thermal/stability qualification |
| CPU/GPU stress suites | Not implemented | Not implemented | Requires dedicated workloads and safe temperature/load design |
| Windows Configuration Manager problem codes | Implemented where WMI exposes `ConfigManagerErrorCode` | N/A | Verify code and device status in Device Manager |
| Linux PCI driver mapping | N/A | Partial via `lspci -nnk` | Candidate kernel module does not prove a fault |
| SMART storage health | Implemented with `smartmontools` | Implemented with `smartmontools` | Controller, bridge and permissions affect access |
| Endurance threshold | Implemented | Implemented | Only applies to defensible remaining-endurance estimate |
| Temperatures/fans | Partial | Partial | ACPI/hwmon sensor labels may not map to component temperatures |
| Battery health | Partial | Partial | Requires paired design and full-charge capacity values |
| USB/display/network inventory | Partial | Partial | Enumeration and link status do not verify physical connectors or cables |
| BIOS/UEFI/TPM/Secure Boot | Partial | Partial | Firmware mode and access vary |
| Update discovery/install | WinGet and Windows Update handoff; selective app update | APT/DNF/Pacman/Zypper plus optional Flatpak/Snap | Explicit confirmation, fresh discovery recheck and system permissions apply |
| Repair Center | Fixed allow-listed actions | Fixed allow-listed actions | No general repair script or arbitrary command execution |
| Recovery-image hash | Implemented (SHA-256) | Implemented (SHA-256) | Expected digest must come from an authenticated publisher source |
| Removable USB inventory | Partial via `Get-Disk` | Partial via `lsblk` | Inventory only; no raw USB writer |
| Bootable USB creation | Guided only | Guided only | External media writer still requires user target verification; CrossDiag does not write the device |
| Firmware flashing / BIOS setting writes | Not implemented | Not implemented | Vendor-specific and high risk |
| Technician cases / scheduled SMART scans | Local SQLite / in-process worker | Local SQLite / in-process worker | Scheduler runs only while CrossDiag is open; case DB not encrypted |
| PDF report | Print dialog / Save as PDF | Print dialog / Save as PDF | User completes save in system print dialog |
| Fault log review | Windows System event log | Current-boot system journal | Bounded collection; evidence is not root cause |
| Physical port/PSU/cable tests | Not implemented | Not implemented | Requires dedicated equipment or physical inspection |

## Optional external tools

- `smartmontools`: physical storage SMART access.
- `pciutils` (`lspci`): richer Linux PCI device/driver inventory.
- `amd-smi`: AMD GPU metrics on supported Linux systems; install from official AMD/ROCm guidance.
- `intel_gpu_top` (`intel-gpu-tools`): Intel engine counters on supported Linux systems.
- `nvidia-smi`: NVIDIA GPU telemetry.
- WinGet, Flatpak and Snap: package inventory/update discovery where supported.
- `pkexec` plus a PolicyKit authentication agent: elevated Linux update transactions when CrossDiag is not already running as root.

All third-party tools remain separately installed and retain their own licenses and compatibility restrictions.
