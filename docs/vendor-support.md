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

## Complete brand catalogue (v1.0.0-rc.2+, 44 profiles)

Every entry below is matched by the same rules and covered by unit tests
(`vendor_catalogue_covers_every_requested_pc_brand`,
`vendor_catalogue_entries_are_well_formed`). Adding a brand is one table entry in
`src-tauri/src/lib.rs`, not another branch of an `if` chain.

| Group | Vendors |
| --- | --- |
| Global OEMs | HP (incl. Hewlett-Packard, OMEN, Victus), Dell (incl. Alienware), Lenovo (ThinkPad/ThinkBook/IdeaPad/Legion), Acer, ASUS (incl. ASUS-serviced Intel NUC), MSI |
| US OEMs | Apple, Microsoft Surface, Samsung, Razer, Framework, Intel |
| Japanese OEMs | Fujitsu, Dynabook / Toshiba, Panasonic |
| Taiwanese OEMs | GIGABYTE, ASRock, Acer, ASUS, MSI, Biostar |
| Other OEMs | LG, HUAWEI, HONOR, Xiaomi / Redmi, VAIO / Sony, MEDION, CHUWI, Teclast, Infinix, itel |
| Regional brands | Walton (Bangladesh), Positivo (Brazil), Lanix (Mexico) |
| Boutique builders | iBUYPOWER, CyberPowerPC, Falcon Northwest, MAINGEAR, ORIGIN PC, Puget Systems, NZXT |
| Linux-first vendors | System76, TUXEDO (Schenker) |
| ARM single-board computers | Raspberry Pi, PINE64 |

### Motherboard fallback for DIY, whitebox and re-badged systems

Windows reports `System manufacturer`, `To Be Filled By O.E.M.`, `Default string`, `Unknown`,
`OEM` or an empty string on many assembled and re-badged machines. On those systems CrossDiag
now reads the **board vendor** (`Win32_BaseBoard` on Windows, `/sys/class/dmi/id/board_vendor`
on Linux) and matches that instead. The result names the board vendor and states that the match
came from the motherboard, for example:

> Matched through the motherboard vendor 'Gigabyte Technology Co., Ltd.' because the system
> manufacturer field is a placeholder. Desktop and custom builds use the motherboard vendor
> firmware.

A real OEM manufacturer always wins over the board vendor: a Lenovo laptop whose mainboard was
built by Compal is still reported as Lenovo.

### Model family detection order

The first matching family keyword wins, so desktop board entries list the chipset before the
marketing line (`b650` before `aorus`). This keeps `B650 AORUS ELITE` reported as a B650 board
while `AORUS 15` is still reported as an AORUS laptop.

### Honest limits of this catalogue

- **Windows 10/11 x64 is the supported target.** Every x86 OEM and builder entry above runs
  Windows 10 or 11, including Windows 11 24H2/25H2 hardware requirements.
- **Apple silicon is not a Windows target.** Intel Macs can run Windows through Boot Camp;
  M-series Macs run Windows only inside a virtual machine, where sensor and driver readings are
  not representative of the hardware. The entry says so instead of pretending.
- **Raspberry Pi and PINE64 are ARM single-board computers.** Windows 10/11 does not run natively
  on them. CrossDiag reports only Linux diagnostics for those boards and gives no UEFI key hints.
- **Boutique builders ship retail mainboards.** Their firmware keys and BIOS packages belong to
  the board vendor, which the notes state.
- **Brands without a single global portal** (CHUWI, Teclast, Infinix, itel) intentionally return
  no support URL rather than guessing one. CrossDiag never sends you to a random driver site.
- Regional portals, model names and firmware packages change. The model number printed on the
  device and the manufacturer's official portal remain authoritative.

## Matching rules

- Manufacturer matching uses the OS-reported manufacturer string, normalized to lowercase.
- Product family is detected only when the reported model contains a known family keyword.
- If no vendor match is confident, CrossDiag returns no support URL. It must not choose a random vendor's driver or firmware site.
- Boot-menu and firmware keys are hints, not guarantees. The exact model manual is authoritative.
- CrossDiag does not scrape current driver versions or warranty data in this release.
