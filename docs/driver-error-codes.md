# Driver and device error interpretation — v0.6.0

## Windows Configuration Manager

On Windows, CrossDiag reads `Win32_PnPEntity.ConfigManagerErrorCode` when available and maps selected common values to plain-language descriptions. The original numeric code and device identity are retained in the finding so a technician can verify the result in Device Manager.

| Code | Summary | Recommended next step |
|---:|---|---|
| 1 | Not configured correctly | Review device properties and the matching hardware ID |
| 3 | Driver may be corrupted or resources insufficient | Inspect system resources, logs and official driver source |
| 10 | Device cannot start | Review device events, power/connection and driver history |
| 12 | Insufficient free resources | Check resource conflicts and platform documentation |
| 14 | Restart required | Save work and perform a normal restart |
| 18 | Drivers should be reinstalled | Confirm exact device and OEM driver package |
| 19 | Registry configuration problem | Review recent software/driver changes and supported recovery steps |
| 22 | Device disabled | Verify it should be enabled before changing Device Manager state |
| 28 | Driver not installed | Identify hardware ID and use Windows Update or OEM support |
| 31 | Required driver cannot load | Inspect driver package, signature and device events |
| 32 | Driver service disabled | Verify policy and service configuration before changing it |
| 37 | Driver initialization failed | Review device events and model-specific OEM guidance |
| 39 | Driver may be missing or corrupted | Verify package integrity and official replacement source |
| 43 | Windows stopped the device after reporting a problem | Correlate logs, cable/power conditions and driver changes; code 43 does not identify a unique failed part |
| 45 | Device reported as not connected | Check whether the device is expected to be present |
| 46 | Device unavailable during shutdown | Recheck after a normal boot |
| 48 | Driver blocked for a known compatibility issue | Review Windows compatibility/security details |
| 52 | Driver signature could not be verified | Prefer a properly signed package from Windows Update or the OEM |

Other codes remain visible as unknown codes and must be inspected in Device Manager. This table is a triage aid, not a definitive root-cause analysis. Windows may report different values across OS versions and device classes; confirm with the device's Properties → General → Device status and [Microsoft's official Device Manager error-code reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/device-manager-error-messages).

## Linux PCI driver binding

`lspci -nnk` can show a `Kernel driver in use` and one or more `Kernel modules` candidates. A candidate module without a bound driver is not automatically a fault; some devices are intentionally unused or handled through another subsystem. CrossDiag reports this as an informational finding, not a failed device.
