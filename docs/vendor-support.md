# Vendor support profiles

CrossDiag v0.6.0 provides a reference profile for common PC vendors. These profiles help users find official support and model documentation; they do not execute manufacturer diagnostics, validate firmware compatibility, or prove a hardware component has passed testing.

| Vendor | Official support | Utility guidance | Important caveat |
|---|---|---|---|
| HP | https://support.hp.com/ | HP PC Hardware Diagnostics UEFI / Support Assistant | Match the exact product number; common startup sequence Esc, then F10/F9. |
| Dell | https://www.dell.com/support/home/ | SupportAssist / ePSA | Use the exact Service Tag; common keys F2/F12. |
| Lenovo | https://support.lenovo.com/ | Lenovo Diagnostics / Vantage | ThinkPad, ThinkBook, IdeaPad and Legion keys vary; F1/F2/F12/Novo are common hints. |
| Acer | https://www.acer.com/support | Care Center / supported diagnostics | F2 setup and F12 boot menu are common; boot menu may be disabled. |
| ASUS | https://www.asus.com/support/ | MyASUS / supported diagnostics | F2/Del setup and Esc boot menu are common. |
| MSI | https://www.msi.com/support | MSI Center / model-specific tests | Delete/F11 are common hints; board and laptop procedures differ. |
| Microsoft Surface | https://support.microsoft.com/surface | Surface Diagnostic Toolkit | Use model-specific Surface UEFI/boot instructions. |
| Samsung | https://www.samsung.com/support/ | Samsung Update / Settings | Galaxy Book firmware procedures vary by generation/region. |
| GIGABYTE | https://www.gigabyte.com/Support | GIGABYTE Control Center | Use the exact motherboard model and revision. |
| Apple | https://support.apple.com/ | Apple Diagnostics in macOS recovery | CrossDiag does not replace Apple Diagnostics or modify Apple firmware. |
| HUAWEI | https://consumer.huawei.com/support/ | PC Manager where supported | Match regional MateBook model. |
| Dynabook/Toshiba | https://www.dynabook.com/support | PC Health Monitor on supported systems | Legacy Toshiba and current Dynabook support differ. |
| Xiaomi/Redmi | https://www.mi.com/global/support/ | PC Manager where supported | Use regional product model. |
| HONOR | https://www.honor.com/global/support/ | PC Manager where supported | Use full model identifier. |
| Fujitsu | https://support.ts.fujitsu.com/ | Diagnostics / DeskUpdate | Business model and region matter. |
| Panasonic | https://pc-dl.panasonic.co.jp/itn/support/ | Toughbook diagnostics where supported | Rugged models have model-specific service instructions. |
| Razer | https://mysupport.razer.com/ | Synapse and model support | Use exact Blade model. |
| LG | https://www.lg.com/us/support/ | Update Center / Smart Assistant where supported | Regional Gram support and firmware differ. |
| VAIO/Sony | https://support.us.vaio.com/ | VAIO Care / Update where supported | Current VAIO and legacy Sony support portals differ. |
| MEDION | https://www.medion.com/gb/service/ | Service App / model-specific diagnostics | Use full product/MSN number. |
| Framework | https://frame.work/support | Framework hardware and repair guides | Module and firmware procedures differ by generation. |
| System76 | https://support.system76.com/ | System76 firmware and diagnostic tools | Model-specific Pop!_OS and firmware guidance. |
| ASRock | https://www.asrock.com/support/ | UEFI and motherboard utilities | Verify exact board revision before firmware downloads. |

## Matching rules

- Manufacturer matching uses the OS-reported manufacturer string, normalized to lowercase.
- Product family is detected only when the reported model contains a known family keyword.
- If no vendor match is confident, CrossDiag returns no support URL. It must not choose a random vendor's driver or firmware site.
- Boot-menu and firmware keys are hints, not guarantees. The exact model manual is authoritative.
- CrossDiag does not scrape current driver versions or warranty data in this release.
