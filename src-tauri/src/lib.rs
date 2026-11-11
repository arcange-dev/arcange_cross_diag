use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use sysinfo::{Disks, System};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    name: String,
    mount_point: String,
    file_system: String,
    total_bytes: u64,
    available_bytes: u64,
    removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    app_version: String,
    os_name: String,
    os_version: String,
    kernel_version: String,
    hostname: String,
    manufacturer: String,
    model: String,
    architecture: String,
    cpu_name: String,
    logical_cores: usize,
    physical_cores: Option<usize>,
    total_memory_bytes: u64,
    used_memory_bytes: u64,
    drives: Vec<DriveInfo>,
    collected_at: String,
    capabilities: Vec<String>,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    id: Option<u64>,
    name: String,
    normalized_value: Option<u64>,
    worst_value: Option<u64>,
    threshold: Option<u64>,
    raw_value: Option<u64>,
    when_failed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalDiskHealth {
    device: String,
    device_type: Option<String>,
    protocol: Option<String>,
    model: Option<String>,
    drive_kind: String,
    capacity_bytes: Option<u64>,
    rotation_rate_rpm: Option<u64>,
    smart_passed: Option<bool>,
    status: String,
    status_reason: String,
    #[serde(default)]
    findings: Vec<String>,
    remaining_endurance_percent: Option<f64>,
    endurance_used_percent: Option<f64>,
    endurance_basis: Option<String>,
    temperature_c: Option<f64>,
    available_spare_percent: Option<u64>,
    available_spare_threshold_percent: Option<u64>,
    critical_warning: Option<u64>,
    media_errors: Option<u64>,
    reallocated_sectors: Option<u64>,
    current_pending_sectors: Option<u64>,
    offline_uncorrectable: Option<u64>,
    power_on_hours: Option<u64>,
    smart_attributes: Vec<SmartAttribute>,
    smartctl_exit_code: Option<i32>,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageScan {
    scanned_at: String,
    threshold_percent: u8,
    smartctl_available: bool,
    smartctl_version: Option<String>,
    scan_status: String,
    disks: Vec<PhysicalDiskHealth>,
    limitations: Vec<String>,
}

#[derive(Debug)]
struct CommandOutput {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeToolCheck {
    id: String,
    label: String,
    category: String,
    available: bool,
    purpose: String,
    install_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReadiness {
    app_version: String,
    platform: String,
    architecture: String,
    checked_at: String,
    tools: Vec<RuntimeToolCheck>,
    limitations: Vec<String>,
}

fn runtime_tool(
    id: &str,
    label: &str,
    category: &str,
    purpose: &str,
    install_hint: &str,
) -> RuntimeToolCheck {
    RuntimeToolCheck {
        id: id.into(),
        label: label.into(),
        category: category.into(),
        available: executable_exists(id),
        purpose: purpose.into(),
        install_hint: install_hint.into(),
    }
}

fn linux_package_manager_from_os_release(release: &str) -> Option<&'static str> {
    let lower = release.to_ascii_lowercase();
    let id = lower
        .lines()
        .find_map(|line| line.strip_prefix("id="))
        .unwrap_or_default()
        .trim_matches('"');
    let id_like = lower
        .lines()
        .find_map(|line| line.strip_prefix("id_like="))
        .unwrap_or_default()
        .trim_matches('"');
    let identifiers = format!("{id} {id_like}");
    let tokens = identifiers.split_whitespace().collect::<Vec<_>>();
    if [
        "ubuntu",
        "debian",
        "linuxmint",
        "pop",
        "elementary",
        "kali",
        "raspbian",
    ]
    .iter()
    .any(|item| tokens.contains(item))
    {
        Some("apt-get")
    } else if ["fedora", "rhel", "centos", "rocky", "almalinux", "ol"]
        .iter()
        .any(|item| tokens.contains(item))
    {
        Some("dnf")
    } else if ["arch", "manjaro", "endeavouros"]
        .iter()
        .any(|item| tokens.contains(item))
    {
        Some("pacman")
    } else if ["opensuse", "suse", "sles"]
        .iter()
        .any(|item| tokens.contains(item))
    {
        Some("zypper")
    } else {
        None
    }
}

fn collect_runtime_readiness() -> RuntimeReadiness {
    let tools = if cfg!(target_os = "windows") {
        vec![
            runtime_tool("smartctl", "smartmontools / smartctl", "Storage", "Read supported HDD/SSD SMART and NVMe health attributes.", "Install smartmontools from its official project or run: winget install --id smartmontools.smartmontools -e"),
            runtime_tool("powershell", "Windows PowerShell", "Windows diagnostics", "Windows CIM, event and device-management collection.", "Windows PowerShell is included with supported Windows editions; verify it is on PATH."),
            runtime_tool("winget", "Windows Package Manager", "Application updates", "Discover and install supported application package updates.", "Install or update App Installer from Microsoft Store to obtain winget."),
            runtime_tool("nvidia-smi", "NVIDIA-SMI", "GPU telemetry", "Read supported NVIDIA GPU utilization, temperature, memory and power metrics.", "Install a compatible NVIDIA driver that includes nvidia-smi."),
            runtime_tool("amd-smi", "AMD SMI CLI", "GPU telemetry", "Read supported AMD GPU metrics where the AMD SMI runtime is installed.", "Follow AMD's official AMD SMI compatibility instructions for your GPU and Windows version."),
        ]
    } else if cfg!(target_os = "linux") {
        let mut tools = vec![
            runtime_tool("smartctl", "smartmontools / smartctl", "Storage", "Read supported HDD/SSD SMART and NVMe health attributes.", "Install smartmontools with your distribution package manager."),
            runtime_tool("lspci", "PCI utilities / lspci", "Hardware inventory", "Enumerate PCI devices and identify bound Linux kernel drivers.", "Install pciutils with your distribution package manager."),
            runtime_tool("lsblk", "Block device inventory / lsblk", "Recovery", "Identify block devices and removable media without writing to them.", "Install util-linux; many distributions include lsblk by default."),
            runtime_tool("sensors", "lm-sensors", "Thermal monitoring", "Read temperatures and fan/voltage sensors exposed by supported kernel drivers.", "Install lm-sensors and configure supported sensor drivers."),
            runtime_tool("journalctl", "systemd journal", "Fault analysis", "Read bounded system error events where journal access is permitted.", "Use systemd-journald; some logs require appropriate group membership or elevated access."),
            runtime_tool("pkexec", "PolicyKit / pkexec", "Authorized maintenance", "Request authorization for selected package-manager operations.", "Install and configure PolicyKit for your desktop distribution."),
            runtime_tool("fwupdmgr", "fwupd", "Firmware inventory", "Discover supported firmware metadata; CrossDiag does not flash firmware automatically.", "Install fwupd where supported by your hardware and distribution."),
            runtime_tool("flatpak", "Flatpak", "Application updates", "Discover supported Flatpak application updates.", "Install Flatpak from your distribution repository if you use Flatpak applications."),
            runtime_tool("snap", "Snap", "Application updates", "Discover supported Snap application updates.", "Install snapd from your distribution repository if you use Snap packages."),
            runtime_tool("nvidia-smi", "NVIDIA-SMI", "GPU telemetry", "Read supported NVIDIA GPU telemetry.", "Install a compatible NVIDIA driver package that provides nvidia-smi."),
            runtime_tool("amd-smi", "AMD SMI CLI", "GPU telemetry", "Read supported AMD GPU telemetry.", "Follow AMD's official AMD SMI/ROCm guide for your GPU and distribution."),
            runtime_tool("intel_gpu_top", "Intel GPU Top", "GPU telemetry", "Read Intel GPU engine activity, not a standardized whole-GPU benchmark.", "Install intel-gpu-tools where supported."),
        ];
        if let Some(manager) = linux_package_manager_from_os_release(
            &fs::read_to_string("/etc/os-release").unwrap_or_default(),
        ) {
            let (label, purpose, hint) = match manager {
                "apt-get" => (
                    "APT package manager",
                    "Discover and apply Debian/Ubuntu-family package updates.",
                    "Available on Debian/Ubuntu-family distributions.",
                ),
                "dnf" => (
                    "DNF package manager",
                    "Discover and apply Fedora-family package updates.",
                    "Available on Fedora-family distributions.",
                ),
                "pacman" => (
                    "Pacman package manager",
                    "Discover and apply Arch-family package updates.",
                    "Available on Arch-family distributions.",
                ),
                "zypper" => (
                    "Zypper package manager",
                    "Discover and apply openSUSE-family package updates.",
                    "Available on openSUSE-family distributions.",
                ),
                _ => (
                    "Distribution package manager",
                    "Manage operating-system package updates.",
                    "Use the package manager documented by your distribution.",
                ),
            };
            tools.push(runtime_tool(
                manager,
                label,
                "Operating-system updates",
                purpose,
                hint,
            ));
        }
        tools
    } else {
        Vec::new()
    };

    RuntimeReadiness {
        app_version: env!("CARGO_PKG_VERSION").into(),
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        checked_at: timestamp_string(),
        tools,
        limitations: vec![
            "Tool presence only confirms that an executable is on PATH; it does not guarantee device access or compatible hardware.".into(),
            "Permissions, drivers, firmware, hardware support and distribution versions affect diagnostic availability.".into(),
            "This check is read-only and never installs packages, updates firmware or repairs the system.".into(),
        ],
    }
}

#[derive(Debug, Clone)]
struct ScannedDevice {
    name: String,
    device_type: Option<String>,
    protocol: Option<String>,
}

fn string_or_unknown(value: Option<String>) -> String {
    value
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Not reported by OS".to_string())
}

fn to_drive_info(disk: &sysinfo::Disk) -> DriveInfo {
    // Mounted filesystems are not guaranteed to map one-to-one to physical drives.
    DriveInfo {
        name: disk.name().to_string_lossy().to_string(),
        mount_point: disk.mount_point().to_string_lossy().to_string(),
        file_system: disk.file_system().to_string_lossy().to_string(),
        total_bytes: disk.total_space(),
        available_bytes: disk.available_space(),
        removable: disk.is_removable(),
    }
}

fn get_system_snapshot_blocking() -> Result<SystemSnapshot, String> {
    let mut system = System::new_all();
    system.refresh_all();

    let cpu_name = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Not reported by OS".to_string());

    let disks = Disks::new_with_refreshed_list();
    let drive_info = disks.list().iter().map(to_drive_info).collect::<Vec<_>>();
    let (manufacturer, model) = platform_computer_identity();

    Ok(SystemSnapshot {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os_name: string_or_unknown(System::name()),
        os_version: string_or_unknown(System::os_version()),
        kernel_version: string_or_unknown(System::kernel_version()),
        hostname: string_or_unknown(System::host_name()),
        manufacturer,
        model,
        architecture: std::env::consts::ARCH.to_string(),
        cpu_name,
        logical_cores: system.cpus().len(),
        physical_cores: system.physical_core_count(),
        total_memory_bytes: system.total_memory(),
        used_memory_bytes: system.used_memory(),
        drives: drive_info,
        collected_at: timestamp_string(),
        capabilities: vec![
            "Operating-system, kernel and architecture information".into(),
            "CPU brand and logical processor count when exposed by the OS".into(),
            "System memory total and current use".into(),
            "Mounted volume capacity and available-space inventory".into(),
        ],
        limitations: vec![
            "SMART data is collected separately through smartmontools when installed".into(),
            "Mounted volumes are not guaranteed to correspond one-to-one with physical drives"
                .into(),
            "Temperature and fan availability depends on hardware and operating-system drivers"
                .into(),
            "Physical connector condition cannot be confirmed by software enumeration alone".into(),
        ],
    })
}

/// Run a fixed executable with fixed/validated arguments and a hard timeout.
/// Output is drained on separate threads to avoid blocking on full stdout/stderr pipes.
fn run_command(
    program: &str,
    args: &[String],
    timeout_seconds: u64,
) -> Result<CommandOutput, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not start {program}: {error}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = thread::spawn(move || read_pipe(stdout));
    let stderr_reader = thread::spawn(move || read_pipe(stderr));

    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(100)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!(
                    "{program} did not finish within {timeout_seconds} seconds"
                ));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!("Could not wait for {program}: {error}"));
            }
        }
    };

    let stdout = stdout_reader
        .join()
        .map_err(|_| format!("Could not read {program} output"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| format!("Could not read {program} error output"))??;

    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code: status.code(),
    })
}

fn read_pipe<R: Read>(pipe: Option<R>) -> Result<String, String> {
    const MAX_COMMAND_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
    let Some(mut pipe) = pipe else {
        return Ok(String::new());
    };
    // Native Windows utilities can emit text in the active OEM code page, and
    // Linux tools can include non-UTF-8 device labels. Preserve readable output
    // with replacement characters instead of failing the entire diagnostic.
    // Continue draining after the cap is reached so a child process cannot block
    // forever because its stdout/stderr pipe is no longer being consumed.
    let mut bytes = Vec::with_capacity(8192);
    let mut chunk = [0_u8; 8192];
    let mut total_bytes = 0_usize;
    let mut exceeded_limit = false;
    loop {
        let count = pipe
            .read(&mut chunk)
            .map_err(|error| format!("Unable to read command output: {error}"))?;
        if count == 0 {
            break;
        }
        total_bytes = total_bytes.saturating_add(count);
        if bytes.len() < MAX_COMMAND_OUTPUT_BYTES {
            let remaining = MAX_COMMAND_OUTPUT_BYTES - bytes.len();
            bytes.extend_from_slice(&chunk[..count.min(remaining)]);
        }
        if total_bytes > MAX_COMMAND_OUTPUT_BYTES {
            exceeded_limit = true;
        }
    }
    if exceeded_limit {
        return Err(format!(
            "Command output exceeded the {MAX_COMMAND_OUTPUT_BYTES}-byte safety limit"
        ));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn timestamp_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| {
        value
            .as_u64()
            .or_else(|| {
                value
                    .as_i64()
                    .filter(|number| *number >= 0)
                    .map(|number| number as u64)
            })
            .or_else(|| value.as_str().and_then(|text| text.parse::<u64>().ok()))
    })
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_u64().map(|number| number as f64))
            .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
    })
}

fn json_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_scanned_devices(output: &str) -> Vec<ScannedDevice> {
    let mut devices = Vec::new();

    // If the output is valid JSON, trust its schema even when it contains no devices.
    // Falling back to line parsing in that case could misread JSON syntax as device paths.
    if let Ok(value) = serde_json::from_str::<Value>(output) {
        if let Some(items) = value.get("devices").and_then(Value::as_array) {
            for item in items {
                let Some(name) = non_empty_string(item.get("name")) else {
                    continue;
                };
                if !valid_device_name(&name) {
                    continue;
                }
                devices.push(ScannedDevice {
                    name,
                    device_type: non_empty_string(item.get("type")),
                    protocol: non_empty_string(item.get("protocol")),
                });
            }
        }
    } else {
        // Text fallback is only used when smartctl did not return valid JSON.
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("smartctl") {
                continue;
            }
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let Some(name) = fields.first().map(|field| field.trim_matches('"')) else {
                continue;
            };
            if !valid_device_name(name) {
                continue;
            }
            let device_type = fields
                .windows(2)
                .find(|pair| pair[0] == "-d")
                .map(|pair| pair[1].trim_matches('"').to_string());
            devices.push(ScannedDevice {
                name: name.to_string(),
                device_type,
                protocol: None,
            });
        }
    }

    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices.dedup_by(|a, b| a.name == b.name);
    devices
}

fn valid_device_name(name: &str) -> bool {
    !name.trim().is_empty()
        && !name.starts_with('-')
        && !name.contains('\0')
        && !name.contains('\n')
        && !name.contains('\r')
        && !name.chars().any(char::is_control)
        && name.len() < 512
}

fn safe_device_type(device_type: &str) -> bool {
    !device_type.is_empty()
        && device_type.len() < 80
        && device_type.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ',' | '.' | '+')
        })
}

fn attribute_by_names<'a>(
    attributes: &'a [SmartAttribute],
    names: &[&str],
) -> Option<&'a SmartAttribute> {
    attributes.iter().find(|attribute| {
        let normalized = attribute.name.to_ascii_lowercase();
        names
            .iter()
            .any(|name| normalized == name.to_ascii_lowercase())
    })
}

fn parse_attributes(value: &Value) -> Vec<SmartAttribute> {
    value
        .get("ata_smart_attributes")
        .and_then(|attributes| attributes.get("table"))
        .and_then(Value::as_array)
        .map(|table| {
            table
                .iter()
                .filter_map(|item| {
                    let name = non_empty_string(item.get("name"))?;
                    Some(SmartAttribute {
                        id: json_u64(item.get("id")),
                        name,
                        normalized_value: json_u64(item.get("value")),
                        worst_value: json_u64(item.get("worst")),
                        threshold: json_u64(item.get("thresh")),
                        raw_value: json_u64(item.get("raw").and_then(|raw| raw.get("value"))),
                        when_failed: non_empty_string(item.get("when_failed")),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn collect_disk_findings(
    smart_passed: Option<bool>,
    critical_warning: Option<u64>,
    media_errors: Option<u64>,
    spare: Option<u64>,
    spare_threshold: Option<u64>,
    reallocated: Option<u64>,
    pending: Option<u64>,
    uncorrectable: Option<u64>,
    remaining_endurance: Option<f64>,
    threshold: u8,
) -> Vec<String> {
    let mut findings = Vec::new();

    if smart_passed == Some(false) {
        findings.push("Drive firmware reports a failed SMART overall-health check.".into());
    }

    if let Some(flags) = critical_warning {
        let warning_flags = [
            (
                1_u64 << 0,
                "Available spare capacity is below the drive-reported threshold.",
            ),
            (
                1_u64 << 1,
                "The drive reports that a temperature threshold was exceeded.",
            ),
            (1_u64 << 2, "The drive reports degraded reliability."),
            (
                1_u64 << 3,
                "The drive reports that media has been placed in read-only mode.",
            ),
            (
                1_u64 << 4,
                "The drive reports a volatile-memory backup failure.",
            ),
            (
                1_u64 << 5,
                "The drive reports that its persistent-memory region is read-only.",
            ),
        ];
        for (mask, description) in warning_flags {
            if flags & mask != 0 {
                findings.push(description.into());
            }
        }
        let unknown_flags = flags & !0b11_1111;
        if unknown_flags != 0 {
            findings.push(format!(
                "The drive reports unrecognized NVMe critical-warning bits: 0x{unknown_flags:X}."
            ));
        }
    }

    if let Some(errors) = media_errors.filter(|value| *value > 0) {
        findings.push(format!(
            "The drive reports {errors} NVMe media/data-integrity error(s)."
        ));
    }
    match (spare, spare_threshold) {
        (Some(available), Some(minimum)) if available <= minimum => findings.push(format!(
            "Available spare capacity is {available}%, at or below the drive-reported {minimum}% threshold."
        )),
        (Some(0), _) => findings.push("Available spare capacity is reported as 0%.".into()),
        _ => {}
    }
    if let Some(value) = reallocated.filter(|value| *value > 0) {
        findings.push(format!(
            "The drive reports {value} reallocated sector(s) or events."
        ));
    }
    if let Some(value) = pending.filter(|value| *value > 0) {
        findings.push(format!("The drive reports {value} pending sector(s)."));
    }
    if let Some(value) = uncorrectable.filter(|value| *value > 0) {
        findings.push(format!(
            "The drive reports {value} offline-uncorrectable sector(s)."
        ));
    }
    if let Some(remaining) = remaining_endurance.filter(|value| *value <= f64::from(threshold)) {
        findings.push(format!(
            "Estimated remaining endurance is {remaining:.1}%, at or below the configured {threshold}% policy threshold."
        ));
    }

    if smart_passed.is_none() && findings.is_empty() {
        findings.push("The device did not provide a SMART overall-health pass/fail result.".into());
    }

    findings
}

fn evaluate_disk_status(
    smart_passed: Option<bool>,
    critical_warning: Option<u64>,
    media_errors: Option<u64>,
    spare: Option<u64>,
    spare_threshold: Option<u64>,
    reallocated: Option<u64>,
    pending: Option<u64>,
    uncorrectable: Option<u64>,
    remaining_endurance: Option<f64>,
    threshold: u8,
) -> (String, String) {
    if smart_passed == Some(false) || critical_warning.is_some_and(|value| value & (1 << 3) != 0) {
        return (
            "failed".into(),
            "The drive reports a failed SMART health check or the NVMe media read-only warning flag.".into(),
        );
    }
    if critical_warning.is_some_and(|value| value != 0) {
        return (
            "attention".into(),
            "The drive reports one or more NVMe critical-warning flags. Review the detailed SMART data.".into(),
        );
    }
    if remaining_endurance.is_some_and(|value| value <= f64::from(threshold))
        || media_errors.is_some_and(|value| value > 0)
        || spare.is_some_and(|available| available == 0)
        || spare
            .zip(spare_threshold)
            .is_some_and(|(available, minimum)| available <= minimum)
        || reallocated.is_some_and(|value| value > 0)
        || pending.is_some_and(|value| value > 0)
        || uncorrectable.is_some_and(|value| value > 0)
    {
        return (
            "attention".into(),
            "One or more reported wear, spare-capacity or media-error indicators need technician review.".into(),
        );
    }
    if smart_passed == Some(true) {
        return (
            "healthy".into(),
            "The drive reports a passing SMART overall-health result. This is not a guarantee against future failure.".into(),
        );
    }
    (
        "unknown".into(),
        "The device was detected, but it did not provide enough supported SMART health evidence for a verdict.".into(),
    )
}

fn parse_smart_report(
    device: &ScannedDevice,
    output: &str,
    exit_code: Option<i32>,
    threshold: u8,
) -> PhysicalDiskHealth {
    let parsed = serde_json::from_str::<Value>(output).ok();
    let Some(value) = parsed else {
        return PhysicalDiskHealth {
            device: device.name.clone(),
            device_type: device.device_type.clone(),
            protocol: device.protocol.clone(),
            model: None,
            drive_kind: "Unknown".into(),
            capacity_bytes: None,
            rotation_rate_rpm: None,
            smart_passed: None,
            status: "unknown".into(),
            status_reason: "smartctl did not return a parseable JSON health report for this device.".into(),
            findings: vec!["The SMART report could not be parsed.".into()],
            remaining_endurance_percent: None,
            endurance_used_percent: None,
            endurance_basis: None,
            temperature_c: None,
            available_spare_percent: None,
            available_spare_threshold_percent: None,
            critical_warning: None,
            media_errors: None,
            reallocated_sectors: None,
            current_pending_sectors: None,
            offline_uncorrectable: None,
            power_on_hours: None,
            smart_attributes: Vec::new(),
            smartctl_exit_code: exit_code,
            limitations: vec!["The raw SMART report could not be parsed; review smartmontools output and version.".into()],
        };
    };

    let attributes = parse_attributes(&value);
    let nvme = value.get("nvme_smart_health_information_log");
    let rotation_rate_rpm = json_u64(value.get("rotation_rate"));
    let rotation_rate_text = non_empty_string(value.get("rotation_rate")).unwrap_or_default();
    let rotation_reports_solid_state = rotation_rate_text
        .to_ascii_lowercase()
        .contains("solid state");
    let protocol = device.protocol.clone().or_else(|| {
        non_empty_string(
            value
                .get("device")
                .and_then(|device| device.get("protocol")),
        )
    });
    let protocol_is_nvme = protocol
        .as_deref()
        .is_some_and(|protocol| protocol.eq_ignore_ascii_case("NVMe"))
        || device
            .device_type
            .as_deref()
            .is_some_and(|kind| kind.to_ascii_lowercase().contains("nvme"));
    let drive_kind =
        if protocol_is_nvme || rotation_rate_rpm == Some(0) || rotation_reports_solid_state {
            "SSD".to_string()
        } else if rotation_rate_rpm.is_some_and(|rate| rate > 0) {
            "HDD".to_string()
        } else {
            "Unknown".to_string()
        };

    let smart_passed = json_bool(
        value
            .get("smart_status")
            .and_then(|status| status.get("passed")),
    );
    // smartmontools 7.5 exposes a protocol-neutral endurance_used value.
    // Older versions/devices may only expose NVMe Percentage Used in the health log.
    let endurance_used_percent = json_f64(
        value
            .get("endurance_used")
            .and_then(|metric| metric.get("current_percent")),
    )
    .or_else(|| json_f64(nvme.and_then(|log| log.get("percentage_used"))))
    .filter(|used| used.is_finite() && *used >= 0.0);
    let mut remaining_endurance_percent =
        endurance_used_percent.map(|used| (100.0 - used).clamp(0.0, 100.0));
    let mut endurance_basis = endurance_used_percent.map(|_| {
        "SMART endurance_used/current_percent (remaining estimate = 100 minus endurance used)"
            .to_string()
    });

    // A small, explicit allowlist of SSD attributes whose names themselves report remaining life.
    // Generic wear-leveling counters are intentionally excluded because their raw units vary by vendor.
    if remaining_endurance_percent.is_none() {
        if let Some(attribute) = attribute_by_names(
            &attributes,
            &[
                "Percent_Lifetime_Remain",
                "SSD_Life_Left",
                "Lifetime_Left",
                "Remaining_Lifetime_Perc",
            ],
        ) {
            if let Some(raw) = attribute.raw_value.filter(|value| *value <= 100) {
                remaining_endurance_percent = Some(raw as f64);
                endurance_basis = Some(format!(
                    "Vendor-specific SMART attribute {} raw value",
                    attribute.name
                ));
            }
        }
        if remaining_endurance_percent.is_none() {
            if let Some(attribute) = attribute_by_names(&attributes, &["Media_Wearout_Indicator"]) {
                if let Some(normalized) = attribute.normalized_value.filter(|value| *value <= 100) {
                    remaining_endurance_percent = Some(normalized as f64);
                    endurance_basis = Some(format!(
                        "Vendor-specific SMART normalized attribute {}",
                        attribute.name
                    ));
                }
            }
        }
    }

    let temperature_c = json_f64(
        value
            .get("temperature")
            .and_then(|temperature| temperature.get("current")),
    )
    .or_else(|| json_f64(nvme.and_then(|log| log.get("temperature"))));
    let available_spare_percent = json_u64(
        value
            .get("spare_available")
            .and_then(|metric| metric.get("current_percent")),
    )
    .or_else(|| json_u64(nvme.and_then(|log| log.get("available_spare"))));
    let available_spare_threshold_percent = json_u64(
        value
            .get("spare_available")
            .and_then(|metric| metric.get("threshold_percent")),
    )
    .or_else(|| json_u64(nvme.and_then(|log| log.get("available_spare_threshold"))));
    let critical_warning = json_u64(nvme.and_then(|log| log.get("critical_warning")));
    let media_errors = json_u64(nvme.and_then(|log| log.get("media_errors")));

    let reallocated_sectors = attribute_by_names(
        &attributes,
        &["Reallocated_Sector_Ct", "Reallocated_Event_Count"],
    )
    .and_then(|attribute| attribute.raw_value);
    let current_pending_sectors = attribute_by_names(
        &attributes,
        &["Current_Pending_Sector", "Current_Pending_Sector_Count"],
    )
    .and_then(|attribute| attribute.raw_value);
    let offline_uncorrectable = attribute_by_names(
        &attributes,
        &["Offline_Uncorrectable", "Offline_Uncorrectable_Sector_Ct"],
    )
    .and_then(|attribute| attribute.raw_value);
    let power_on_hours = json_u64(
        value
            .get("power_on_time")
            .and_then(|time| time.get("hours")),
    )
    .or_else(|| json_u64(nvme.and_then(|log| log.get("power_on_hours"))))
    .or_else(|| {
        attribute_by_names(&attributes, &["Power_On_Hours"])
            .and_then(|attribute| attribute.raw_value)
    });

    let (status, status_reason) = evaluate_disk_status(
        smart_passed,
        critical_warning,
        media_errors,
        available_spare_percent,
        available_spare_threshold_percent,
        reallocated_sectors,
        current_pending_sectors,
        offline_uncorrectable,
        remaining_endurance_percent,
        threshold,
    );
    let findings = collect_disk_findings(
        smart_passed,
        critical_warning,
        media_errors,
        available_spare_percent,
        available_spare_threshold_percent,
        reallocated_sectors,
        current_pending_sectors,
        offline_uncorrectable,
        remaining_endurance_percent,
        threshold,
    );

    let mut limitations = Vec::new();
    if remaining_endurance_percent.is_none() {
        limitations.push("The device did not expose a recognized remaining-endurance percentage; no percentage has been invented.".into());
    }
    if smart_passed.is_none() {
        limitations
            .push("The device did not expose a SMART overall-health pass/fail field.".into());
    }
    if exit_code.unwrap_or_default() != 0 {
        limitations.push(format!("smartctl returned exit code {}; the report was parsed but may contain warnings or access limitations.", exit_code.unwrap_or_default()));
    }
    limitations.push(
        "SMART is a predictive indicator, not a guarantee or a substitute for a backup.".into(),
    );

    PhysicalDiskHealth {
        device: device.name.clone(),
        device_type: device.device_type.clone(),
        protocol,
        model: non_empty_string(value.get("model_name"))
            .or_else(|| non_empty_string(value.get("model_family")))
            .or_else(|| non_empty_string(value.get("scsi_model_name"))),
        drive_kind,
        capacity_bytes: json_u64(
            value
                .get("user_capacity")
                .and_then(|capacity| capacity.get("bytes")),
        ),
        rotation_rate_rpm,
        smart_passed,
        status,
        status_reason,
        findings,
        remaining_endurance_percent,
        endurance_used_percent,
        endurance_basis,
        temperature_c,
        available_spare_percent,
        available_spare_threshold_percent,
        critical_warning,
        media_errors,
        reallocated_sectors,
        current_pending_sectors,
        offline_uncorrectable,
        power_on_hours,
        smart_attributes: attributes,
        smartctl_exit_code: exit_code,
        limitations,
    }
}

fn legacy_storage_history_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve CrossDiag app-data folder: {error}"))?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create CrossDiag app-data folder: {error}"))?;
    Ok(directory.join("storage-history.json"))
}

fn insert_storage_history(connection: &mut Connection, scan: &StorageScan) -> Result<(), String> {
    let scan_json = serde_json::to_string(scan)
        .map_err(|error| format!("Could not serialize storage scan: {error}"))?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Could not start storage-history transaction: {error}"))?;
    transaction
        .execute(
            "INSERT INTO storage_scans (scanned_at, scan_json) VALUES (?1, ?2)",
            params![&scan.scanned_at, &scan_json],
        )
        .map_err(|error| format!("Could not append storage scan history: {error}"))?;
    transaction.execute(
        "DELETE FROM storage_scans WHERE id NOT IN (SELECT id FROM storage_scans ORDER BY id DESC LIMIT 100)",
        [],
    ).map_err(|error| format!("Could not apply storage-history retention: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not commit storage-history update: {error}"))
}

fn read_storage_history(connection: &Connection) -> Result<Vec<StorageScan>, String> {
    let mut statement = connection
        .prepare("SELECT scan_json FROM storage_scans ORDER BY id ASC")
        .map_err(|error| format!("Could not prepare storage-history query: {error}"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not query storage history: {error}"))?;
    let serialized = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read storage-history rows: {error}"))?;
    serialized
        .into_iter()
        .map(|json| {
            serde_json::from_str::<StorageScan>(&json)
                .map_err(|error| format!("A saved storage-history record is malformed: {error}"))
        })
        .collect()
}

fn migrate_legacy_storage_history_file(
    connection: &mut Connection,
    legacy_path: &std::path::Path,
) -> Result<(), String> {
    // IMMEDIATE obtains the SQLite write reservation before checking the marker,
    // preventing two first-run callers from importing the same legacy JSON twice.
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start storage-history migration: {error}"))?;
    let migrated: Option<String> = transaction
        .query_row(
            "SELECT value FROM app_settings WHERE key='storage_history_json_migrated'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not inspect storage-history migration state: {error}"))?;
    if migrated.as_deref() == Some("true") {
        transaction.commit().map_err(|error| {
            format!("Could not finish storage-history migration check: {error}")
        })?;
        return Ok(());
    }

    let legacy_scans = if legacy_path.exists() {
        const MAX_LEGACY_HISTORY_BYTES: u64 = 16 * 1024 * 1024;
        let mut legacy_file = fs::File::open(legacy_path)
            .map_err(|error| format!("Could not open legacy storage history: {error}"))?;
        let mut content_bytes = Vec::new();
        legacy_file
            .by_ref()
            .take(MAX_LEGACY_HISTORY_BYTES + 1)
            .read_to_end(&mut content_bytes)
            .map_err(|error| format!("Could not read legacy storage history: {error}"))?;
        if content_bytes.len() as u64 > MAX_LEGACY_HISTORY_BYTES {
            return Err(format!("Legacy storage history exceeds the {MAX_LEGACY_HISTORY_BYTES}-byte migration safety limit; the original file was not changed."));
        }
        let content = String::from_utf8(content_bytes).map_err(|error| {
            format!("Legacy storage history is not valid UTF-8; it has not been changed: {error}")
        })?;
        serde_json::from_str::<Vec<StorageScan>>(&content).map_err(|error| {
            format!("Legacy storage history is malformed; it has not been changed: {error}")
        })?
    } else {
        Vec::new()
    };

    let first_retained = legacy_scans.len().saturating_sub(100);
    for scan in &legacy_scans[first_retained..] {
        let scan_json = serde_json::to_string(scan)
            .map_err(|error| format!("Could not serialize legacy storage scan: {error}"))?;
        transaction
            .execute(
                "INSERT INTO storage_scans (scanned_at, scan_json) VALUES (?1, ?2)",
                params![&scan.scanned_at, &scan_json],
            )
            .map_err(|error| format!("Could not migrate legacy storage scan: {error}"))?;
    }
    transaction.execute(
        "INSERT INTO app_settings (key,value) VALUES ('storage_history_json_migrated','true') ON CONFLICT(key) DO UPDATE SET value='true'",
        [],
    ).map_err(|error| format!("Could not record storage-history migration: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not commit storage-history migration: {error}"))
}

fn migrate_legacy_storage_history(
    app: &AppHandle,
    connection: &mut Connection,
) -> Result<(), String> {
    let legacy_path = legacy_storage_history_path(app)?;
    migrate_legacy_storage_history_file(connection, &legacy_path)
}

fn append_storage_history(app: &AppHandle, scan: &StorageScan) -> Result<(), String> {
    let mut connection = open_case_database(app)?;
    migrate_legacy_storage_history(app, &mut connection)?;
    insert_storage_history(&mut connection, scan)
}

#[tauri::command]
async fn scan_storage_health(
    app: AppHandle,
    threshold_percent: Option<u8>,
) -> Result<StorageScan, String> {
    tauri::async_runtime::spawn_blocking(move || {
        scan_storage_health_blocking(app, threshold_percent)
    })
    .await
    .map_err(|error| format!("Storage scan worker failed: {error}"))?
}

fn scan_storage_health_blocking(
    app: AppHandle,
    threshold_percent: Option<u8>,
) -> Result<StorageScan, String> {
    let threshold = threshold_percent.unwrap_or(90).min(100);
    let scanned_at = timestamp_string();
    let mut scan = StorageScan {
        scanned_at,
        threshold_percent: threshold,
        smartctl_available: false,
        smartctl_version: None,
        scan_status: "tool_missing".into(),
        disks: Vec::new(),
        limitations: Vec::new(),
    };

    let version = match run_command("smartctl", &["--version".into()], 5) {
        Ok(output) => output,
        Err(error) => {
            scan.limitations.push("smartmontools is not installed or could not be started. Install smartmontools and run the scan again.".into());
            scan.limitations.push(error);
            if let Err(history_error) = append_storage_history(&app, &scan) {
                scan.limitations.push(format!(
                    "The scan result could not be saved to local history: {history_error}"
                ));
            }
            return Ok(scan);
        }
    };
    scan.smartctl_available = true;
    scan.smartctl_version = version
        .stdout
        .lines()
        .find(|line| line.to_ascii_lowercase().contains("smartctl"))
        .map(str::trim)
        .map(ToOwned::to_owned);

    let scan_output = match run_command("smartctl", &["--scan-open".into(), "-j".into()], 20) {
        Ok(output) => output,
        Err(error) => {
            scan.scan_status = "scan_error".into();
            scan.limitations.push(error);
            scan.limitations.push("Device enumeration did not finish. Some devices may require administrator/root privileges or a compatible controller driver.".into());
            if let Err(history_error) = append_storage_history(&app, &scan) {
                scan.limitations.push(format!(
                    "The scan result could not be saved to local history: {history_error}"
                ));
            }
            return Ok(scan);
        }
    };

    if scan_output.exit_code.unwrap_or_default() != 0 {
        scan.limitations.push(format!(
            "smartctl device enumeration returned exit code {}.",
            scan_output.exit_code.unwrap_or_default()
        ));
    }
    if !scan_output.stderr.trim().is_empty() {
        scan.limitations.push(format!(
            "smartctl scan message: {}",
            scan_output.stderr.trim()
        ));
    }

    let devices = parse_scanned_devices(&scan_output.stdout);
    if devices.is_empty() {
        scan.scan_status = "no_devices".into();
        scan.limitations.push("smartctl did not enumerate any SMART-capable physical devices. USB bridges, RAID controllers, permissions or unsupported devices can hide drives.".into());
        if let Err(history_error) = append_storage_history(&app, &scan) {
            scan.limitations.push(format!(
                "The scan result could not be saved to local history: {history_error}"
            ));
        }
        return Ok(scan);
    }

    for device in devices {
        let mut args = vec!["-a".to_string(), "-j".to_string()];
        if let Some(device_type) = device
            .device_type
            .as_deref()
            .filter(|kind| safe_device_type(kind))
        {
            args.push("-d".into());
            args.push(device_type.to_string());
        }
        args.push(device.name.clone());

        match run_command("smartctl", &args, 20) {
            Ok(output) => {
                let mut disk = parse_smart_report(&device, &output.stdout, output.exit_code, threshold);
                if !output.stderr.trim().is_empty() {
                    disk.limitations.push(format!("smartctl message: {}", output.stderr.trim()));
                }
                scan.disks.push(disk);
            }
            Err(error) => scan.disks.push(PhysicalDiskHealth {
                device: device.name,
                device_type: device.device_type,
                protocol: device.protocol,
                model: None,
                drive_kind: "Unknown".into(),
                capacity_bytes: None,
                rotation_rate_rpm: None,
                smart_passed: None,
                status: "unknown".into(),
                status_reason: "The drive was enumerated, but its SMART report could not be collected.".into(),
                findings: vec!["The SMART report could not be collected.".into()],
                remaining_endurance_percent: None,
                endurance_used_percent: None,
                endurance_basis: None,
                temperature_c: None,
                available_spare_percent: None,
                available_spare_threshold_percent: None,
                critical_warning: None,
                media_errors: None,
                reallocated_sectors: None,
                current_pending_sectors: None,
                offline_uncorrectable: None,
                power_on_hours: None,
                smart_attributes: Vec::new(),
                smartctl_exit_code: None,
                limitations: vec![error, "Try running CrossDiag with the appropriate read-only device permissions if you are authorized to do so.".into()],
            }),
        }
    }

    scan.scan_status = "completed".into();
    scan.limitations.push("The reported devices are those enumerated by smartmontools; controller and bridge support varies.".into());
    scan.limitations.push("The 90% rule is applied only to a recognized remaining-endurance estimate. Drives without that measurement are not assigned a percentage.".into());
    // History failure must not discard the current diagnostic result.
    if let Err(error) = append_storage_history(&app, &scan) {
        scan.limitations.push(format!(
            "The scan succeeded but local history could not be saved: {error}"
        ));
    }
    Ok(scan)
}

#[tauri::command]
fn get_storage_history(app: AppHandle) -> Result<Vec<StorageScan>, String> {
    let mut connection = open_case_database(&app)?;
    migrate_legacy_storage_history(&app, &mut connection)?;
    read_storage_history(&connection)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSensor {
    name: String,
    category: String,
    value: f64,
    unit: String,
    status: String,
    critical_limit: Option<f64>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    name: String,
    vendor_id: Option<String>,
    device_id: Option<String>,
    driver: Option<String>,
    status: String,
    vram_bytes: Option<u64>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    name: String,
    status: Option<String>,
    charge_percent: Option<f64>,
    health_percent: Option<f64>,
    design_capacity_mwh: Option<u64>,
    full_charge_capacity_mwh: Option<u64>,
    source: String,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAdapterInfo {
    name: String,
    status: Option<String>,
    link_speed: Option<String>,
    mac_address: Option<String>,
    received_bytes: u64,
    transmitted_bytes: u64,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDevice {
    name: String,
    class: Option<String>,
    status: String,
    instance_id: Option<String>,
    problem_code: Option<u32>,
    problem_description: Option<String>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    vendor: Option<String>,
    version: Option<String>,
    release_date: Option<String>,
    mode: Option<String>,
    secure_boot: Option<bool>,
    tpm_detected: Option<bool>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryModule {
    locator: Option<String>,
    manufacturer: Option<String>,
    part_number: Option<String>,
    capacity_bytes: Option<u64>,
    speed_mhz: Option<u64>,
    configured_speed_mhz: Option<u64>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    device: String,
    manufacturer: Option<String>,
    driver: Option<String>,
    version: Option<String>,
    signed: Option<bool>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralInfo {
    category: String,
    name: String,
    status: Option<String>,
    identifier: Option<String>,
    link_speed: Option<String>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    name: String,
    usage_percent: f64,
    frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTelemetry {
    name: String,
    utilization_percent: Option<f64>,
    memory_utilization_percent: Option<f64>,
    memory_total_mib: Option<f64>,
    memory_used_mib: Option<f64>,
    temperature_c: Option<f64>,
    graphics_clock_mhz: Option<f64>,
    power_draw_watts: Option<f64>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSupportProfile {
    vendor: String,
    product_family: Option<String>,
    support_url: Option<String>,
    diagnostic_utility: Option<String>,
    firmware_setup_hint: String,
    boot_menu_hint: String,
    notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverFinding {
    device: String,
    severity: String,
    code: String,
    explanation: String,
    recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDiagnostics {
    scanned_at: String,
    cpu_usage_percent: f64,
    cpu_frequency_mhz: Option<u64>,
    cpu_cores: Vec<CpuCoreInfo>,
    available_memory_bytes: u64,
    memory_modules: Vec<MemoryModule>,
    drivers: Vec<DriverInfo>,
    driver_findings: Vec<DriverFinding>,
    peripherals: Vec<PeripheralInfo>,
    gpus: Vec<GpuDevice>,
    gpu_telemetry: Vec<GpuTelemetry>,
    sensors: Vec<HardwareSensor>,
    batteries: Vec<BatteryInfo>,
    network_adapters: Vec<NetworkAdapterInfo>,
    problem_devices: Vec<ProblemDevice>,
    firmware: FirmwareInfo,
    vendor_support: VendorSupportProfile,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableUpdate {
    name: String,
    version: Option<String>,
    source: String,
    detail: Option<String>,
    #[serde(default)]
    install_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    checked_at: String,
    platform: String,
    package_manager: String,
    status: String,
    update_count: Option<u32>,
    updates: Vec<AvailableUpdate>,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFault {
    timestamp: Option<String>,
    severity: String,
    source: String,
    code: Option<String>,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultReport {
    scanned_at: String,
    platform: String,
    source: String,
    faults: Vec<SystemFault>,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryImageInfo {
    path: String,
    file_name: String,
    size_bytes: u64,
    sha256: String,
    expected_sha256: Option<String>,
    checksum_matches: Option<bool>,
    inspected_at: String,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMediaTarget {
    path: String,
    name: String,
    size_bytes: u64,
    transport: Option<String>,
    removable: bool,
    system_disk: Option<bool>,
    boot_disk: Option<bool>,
    mounted: bool,
    source: String,
}

fn validate_sha256_input(value: Option<String>) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            if normalized.len() != 64
                || !normalized
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                return Err(
                    "Expected SHA-256 must contain exactly 64 hexadecimal characters.".into(),
                );
            }
            Ok(Some(normalized))
        }
    }
}

fn inspect_recovery_image_blocking(
    path: &str,
    expected_sha256: Option<String>,
) -> Result<RecoveryImageInfo, String> {
    let expected_sha256 = validate_sha256_input(expected_sha256)?;
    let supplied = std::path::Path::new(path.trim());
    if path.trim().is_empty() {
        return Err("Select a local ISO or disk-image file first.".into());
    }
    let canonical = supplied
        .canonicalize()
        .map_err(|error| format!("Cannot open the selected image path: {error}"))?;
    let metadata = fs::metadata(&canonical)
        .map_err(|error| format!("Cannot inspect the selected image: {error}"))?;
    if !metadata.is_file() {
        return Err("The selected path is not a regular file.".into());
    }
    if metadata.len() == 0 {
        return Err("The selected image is empty.".into());
    }
    const MAX_IMAGE_SIZE: u64 = 64 * 1024 * 1024 * 1024;
    if metadata.len() > MAX_IMAGE_SIZE {
        return Err(
            "The selected image exceeds the 64 GiB safety limit for in-app hashing.".into(),
        );
    }
    let mut file = fs::File::open(&canonical)
        .map_err(|error| format!("Cannot read selected image: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("Image read failed: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    let sha256 = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let checksum_matches = expected_sha256.as_ref().map(|expected| expected == &sha256);
    Ok(RecoveryImageInfo {
        path: canonical.to_string_lossy().to_string(),
        file_name: canonical.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| "Selected image".into()),
        size_bytes: metadata.len(), sha256, expected_sha256, checksum_matches, inspected_at: timestamp_string(),
        limitations: vec![
            "Hash verification only proves that the local file matches the checksum you supplied; obtain that checksum from the operating-system publisher over an authenticated official channel.".into(),
            "CrossDiag does not write images to USB media. Use a trusted media writer, re-check its target device, and understand that writing erases the selected target.".into(),
            "An image hash does not prove that the image is compatible with a particular computer or that the USB media will boot.".into(),
        ],
    })
}

fn json_flag(value: Option<&Value>) -> bool {
    value
        .map(|item| {
            item.as_bool().unwrap_or_else(|| {
                item.as_u64()
                    .map(|number| number > 0)
                    .or_else(|| {
                        item.as_str()
                            .map(|text| text == "1" || text.eq_ignore_ascii_case("true"))
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn nested_has_mount(value: &Value) -> bool {
    let own = value
        .get("mountpoints")
        .or_else(|| value.get("mountpoint"))
        .map(|mounts| match mounts {
            Value::Array(items) => items
                .iter()
                .any(|item| item.as_str().is_some_and(|text| !text.trim().is_empty())),
            Value::String(text) => !text.trim().is_empty(),
            _ => false,
        })
        .unwrap_or(false);
    own || value
        .get("children")
        .and_then(Value::as_array)
        .is_some_and(|children| children.iter().any(nested_has_mount))
}

fn nested_has_root_mount(value: &Value) -> bool {
    let own = value
        .get("mountpoints")
        .or_else(|| value.get("mountpoint"))
        .map(|mounts| match mounts {
            Value::Array(items) => items.iter().any(|item| item.as_str() == Some("/")),
            Value::String(text) => text == "/",
            _ => false,
        })
        .unwrap_or(false);
    own || value
        .get("children")
        .and_then(Value::as_array)
        .is_some_and(|children| children.iter().any(nested_has_root_mount))
}

fn parse_lsblk_recovery_targets(output: &str) -> Vec<RecoveryMediaTarget> {
    let Ok(root) = serde_json::from_str::<Value>(output) else {
        return Vec::new();
    };
    let Some(devices) = root.get("blockdevices").and_then(Value::as_array) else {
        return Vec::new();
    };
    devices
        .iter()
        .filter_map(|device| {
            if device.get("type").and_then(Value::as_str) != Some("disk") {
                return None;
            }
            let transport = json_string(device.get("tran"));
            let removable = json_flag(device.get("rm"))
                || transport
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case("usb"));
            if !removable {
                return None;
            }
            let name = json_string(device.get("name"))?;
            Some(RecoveryMediaTarget {
                path: if name.starts_with("/dev/") {
                    name.clone()
                } else {
                    format!("/dev/{name}")
                },
                name: json_string(device.get("model")).unwrap_or_else(|| name.clone()),
                size_bytes: json_u64(device.get("size")).unwrap_or(0),
                transport,
                removable: true,
                system_disk: Some(nested_has_root_mount(device)),
                boot_disk: None,
                mounted: nested_has_mount(device),
                source: "lsblk JSON; inventory only".into(),
            })
        })
        .collect()
}

fn list_recovery_media_blocking() -> Result<Vec<RecoveryMediaTarget>, String> {
    #[cfg(target_os = "linux")]
    {
        if !executable_exists("lsblk") {
            return Err(
                "lsblk is not installed; install util-linux to inventory removable media.".into(),
            );
        }
        let args = vec![
            "--json".into(),
            "--bytes".into(),
            "--output".into(),
            "NAME,SIZE,RM,TYPE,MODEL,TRAN,MOUNTPOINTS".into(),
        ];
        let output = run_command("lsblk", &args, 10)?;
        if output.exit_code != Some(0) {
            return Err(format!("lsblk failed: {}", output.stderr.trim()));
        }
        return Ok(parse_lsblk_recovery_targets(&output.stdout));
    }
    #[cfg(target_os = "windows")]
    {
        let script = r#"$ErrorActionPreference='Stop'; $items = @(Get-Disk | Where-Object { $_.BusType -eq 'USB' } | ForEach-Object { $disk=$_; $parts=@(Get-Partition -DiskNumber $disk.Number -ErrorAction SilentlyContinue); $mounted=@($parts | Where-Object { $_.DriveLetter }).Count -gt 0; [pscustomobject]@{ path=('\\.\PhysicalDrive' + $disk.Number); name=$disk.FriendlyName; sizeBytes=[UInt64]$disk.Size; transport=[string]$disk.BusType; removable=$true; systemDisk=[bool]$disk.IsSystem; bootDisk=[bool]$disk.IsBoot; mounted=$mounted } }); $items | ConvertTo-Json -Depth 4 -Compress"#;
        let output = run_command(
            "powershell.exe",
            &[
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                script.into(),
            ],
            20,
        )?;
        if output.exit_code != Some(0) {
            return Err(format!(
                "Unable to inventory USB disks: {}",
                output.stderr.trim()
            ));
        }
        let parsed = serde_json::from_str::<Value>(&output.stdout)
            .map_err(|error| format!("Windows disk inventory returned invalid JSON: {error}"))?;
        let items = parsed.as_array().cloned().unwrap_or_else(|| {
            if parsed.is_object() {
                vec![parsed]
            } else {
                Vec::new()
            }
        });
        return Ok(items
            .iter()
            .filter_map(|item| {
                let path = json_string(item.get("path"))?;
                Some(RecoveryMediaTarget {
                    path,
                    name: json_string(item.get("name"))
                        .unwrap_or_else(|| "USB storage device".into()),
                    size_bytes: json_u64(item.get("sizeBytes")).unwrap_or(0),
                    transport: json_string(item.get("transport")),
                    removable: json_flag(item.get("removable")),
                    system_disk: item.get("systemDisk").and_then(Value::as_bool),
                    boot_disk: item.get("bootDisk").and_then(Value::as_bool),
                    mounted: json_flag(item.get("mounted")),
                    source: "Windows Get-Disk; USB inventory only".into(),
                })
            })
            .collect());
    }
    #[allow(unreachable_code)]
    Ok(Vec::new())
}

#[tauri::command]
async fn inspect_recovery_image(
    path: String,
    expected_sha256: Option<String>,
) -> Result<RecoveryImageInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        inspect_recovery_image_blocking(&path, expected_sha256)
    })
    .await
    .map_err(|error| format!("Image inspection worker failed: {error}"))?
}

#[tauri::command]
async fn list_recovery_media() -> Result<Vec<RecoveryMediaTarget>, String> {
    tauri::async_runtime::spawn_blocking(list_recovery_media_blocking)
        .await
        .map_err(|error| format!("Removable-media inventory worker failed: {error}"))?
}

#[tauri::command]
async fn get_system_snapshot() -> Result<SystemSnapshot, String> {
    tauri::async_runtime::spawn_blocking(get_system_snapshot_blocking)
        .await
        .map_err(|error| format!("System inventory worker failed: {error}"))?
}

#[tauri::command]
fn get_runtime_readiness() -> RuntimeReadiness {
    collect_runtime_readiness()
}

fn platform_computer_identity() -> (String, String) {
    #[cfg(target_os = "windows")]
    {
        let script = "$cs = Get-CimInstance Win32_ComputerSystem; [pscustomobject]@{manufacturer=$cs.Manufacturer; model=$cs.Model} | ConvertTo-Json -Compress";
        if let Ok(output) = run_command(
            "powershell.exe",
            &[
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                script.into(),
            ],
            15,
        ) {
            if let Ok(value) = serde_json::from_str::<Value>(&output.stdout) {
                return (
                    json_string(value.get("manufacturer"))
                        .unwrap_or_else(|| "Not reported by OS".into()),
                    json_string(value.get("model")).unwrap_or_else(|| "Not reported by OS".into()),
                );
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let dmi = std::path::Path::new("/sys/class/dmi/id");
        return (
            read_text(&dmi.join("sys_vendor")).unwrap_or_else(|| "Not reported by OS".into()),
            read_text(&dmi.join("product_name"))
                .or_else(|| read_text(&dmi.join("board_name")))
                .unwrap_or_else(|| "Not reported by OS".into()),
        );
    }
    #[allow(unreachable_code)]
    ("Not reported by OS".into(), "Not reported by OS".into())
}

fn json_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|v| {
        v.as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn json_number(value: Option<&Value>) -> Option<u64> {
    value.and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_i64().filter(|n| *n >= 0).map(|n| n as u64))
    })
}

fn json_float(value: Option<&Value>) -> Option<f64> {
    value.and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|n| n as f64)))
}

fn json_array(value: Option<&Value>) -> Vec<Value> {
    value.and_then(Value::as_array).cloned().unwrap_or_default()
}

fn read_text(path: &std::path::Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_number(path: &std::path::Path) -> Option<u64> {
    read_text(path).and_then(|value| value.parse::<u64>().ok())
}

fn calculate_battery_health(
    full_capacity: Option<u64>,
    design_capacity: Option<u64>,
) -> Option<f64> {
    match (full_capacity, design_capacity) {
        (Some(full), Some(design)) if full > 0 && design > 0 => {
            Some(((full as f64 / design as f64) * 100.0).clamp(0.0, 150.0))
        }
        _ => None,
    }
}

fn executable_exists(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|directory| {
                let candidate = directory.join(program);
                if candidate.is_file() {
                    return true;
                }
                #[cfg(target_os = "windows")]
                {
                    [".exe", ".cmd", ".bat"]
                        .iter()
                        .any(|suffix| directory.join(format!("{program}{suffix}")).is_file())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            })
        })
        .unwrap_or(false)
}

fn parse_optional_metric(value: &str) -> Option<f64> {
    let cleaned = value.trim().trim_end_matches('%').trim();
    if cleaned.is_empty()
        || cleaned.eq_ignore_ascii_case("N/A")
        || cleaned.eq_ignore_ascii_case("[N/A]")
    {
        return None;
    }
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite() && *number >= 0.0)
}

fn parse_nvidia_smi_csv(output: &str) -> Vec<GpuTelemetry> {
    output
        .lines()
        .filter_map(|line| {
            let columns = line.split(',').map(str::trim).collect::<Vec<_>>();
            if columns.len() < 7 || columns[0].is_empty() {
                return None;
            }
            Some(GpuTelemetry {
                name: columns[0].to_string(),
                utilization_percent: parse_optional_metric(columns[1])
                    .filter(|value| *value <= 100.0),
                memory_utilization_percent: parse_optional_metric(columns[2])
                    .filter(|value| *value <= 100.0),
                memory_total_mib: parse_optional_metric(columns[3]),
                memory_used_mib: parse_optional_metric(columns[4]),
                temperature_c: parse_optional_metric(columns[5]),
                graphics_clock_mhz: parse_optional_metric(columns[6]),
                power_draw_watts: columns
                    .get(7)
                    .and_then(|value| parse_optional_metric(value)),
                source: "NVIDIA-SMI read-only telemetry".into(),
            })
        })
        .collect()
}

fn collect_nvidia_gpu_telemetry() -> Vec<GpuTelemetry> {
    #[cfg(target_os = "windows")]
    let program = {
        let standard = std::env::var_os("ProgramFiles")
            .map(std::path::PathBuf::from)
            .map(|path| {
                path.join("NVIDIA Corporation")
                    .join("NVSMI")
                    .join("nvidia-smi.exe")
            });
        standard
            .filter(|path| path.is_file())
            .map(|path| path.to_string_lossy().to_string())
            .or_else(|| executable_exists("nvidia-smi").then(|| "nvidia-smi".to_string()))
    };
    #[cfg(not(target_os = "windows"))]
    let program = executable_exists("nvidia-smi").then(|| "nvidia-smi".to_string());

    let Some(program) = program else {
        return Vec::new();
    };
    let args = vec![
        "--query-gpu=name,utilization.gpu,utilization.memory,memory.total,memory.used,temperature.gpu,clocks.current.graphics,power.draw".to_string(),
        "--format=csv,noheader,nounits".to_string(),
    ];
    match run_command(&program, &args, 5) {
        Ok(output) if output.exit_code == Some(0) => parse_nvidia_smi_csv(&output.stdout),
        _ => Vec::new(),
    }
}

fn normalized_json_key(key: &str) -> String {
    key.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn metric_text_number(text: &str) -> Option<f64> {
    let cleaned = text.trim().replace(',', "");
    if cleaned.is_empty()
        || cleaned.eq_ignore_ascii_case("n/a")
        || cleaned.eq_ignore_ascii_case("not supported")
    {
        return None;
    }
    let start = cleaned.find(|character: char| {
        character.is_ascii_digit() || character == '-' || character == '+'
    })?;
    let numeric = cleaned[start..]
        .chars()
        .take_while(|character| {
            character.is_ascii_digit()
                || *character == '.'
                || *character == '-'
                || *character == '+'
        })
        .collect::<String>();
    numeric
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value >= 0.0)
}

fn metric_value_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .filter(|number| number.is_finite() && *number >= 0.0),
        Value::String(text) => metric_text_number(text),
        Value::Object(map) => {
            for preferred in [
                "current",
                "value",
                "average",
                "avg",
                "percent",
                "temperature",
                "power",
            ] {
                if let Some(found) = map
                    .iter()
                    .find(|(key, _)| normalized_json_key(key) == preferred)
                {
                    if let Some(number) = metric_value_number(found.1) {
                        return Some(number);
                    }
                }
            }
            map.values().find_map(metric_value_number)
        }
        Value::Array(items) => items.iter().find_map(metric_value_number),
        _ => None,
    }
}

fn find_json_metric(value: &Value, aliases: &[&str]) -> Option<f64> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if aliases
                    .iter()
                    .any(|alias| normalized_json_key(alias) == normalized_json_key(key))
                {
                    if let Some(number) = metric_value_number(child) {
                        return Some(number);
                    }
                }
            }
            map.values()
                .find_map(|child| find_json_metric(child, aliases))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| find_json_metric(child, aliases)),
        _ => None,
    }
}

fn find_json_text(value: &Value, aliases: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if aliases
                    .iter()
                    .any(|alias| normalized_json_key(alias) == normalized_json_key(key))
                {
                    if let Some(text) = child
                        .as_str()
                        .map(str::trim)
                        .filter(|text| !text.is_empty() && !text.eq_ignore_ascii_case("n/a"))
                    {
                        return Some(text.to_string());
                    }
                }
            }
            map.values()
                .find_map(|child| find_json_text(child, aliases))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| find_json_text(child, aliases)),
        _ => None,
    }
}

fn find_amd_gpu_nodes(value: &Value, nodes: &mut Vec<(String, Value)>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if normalized_json_key(key) == "gpu" {
                    match child {
                        Value::Object(devices) => {
                            for (device_id, device) in devices {
                                if device.is_object() {
                                    nodes.push((format!("AMD GPU {device_id}"), device.clone()));
                                }
                            }
                        }
                        Value::Array(devices) => {
                            for (index, device) in devices.iter().enumerate() {
                                if device.is_object() {
                                    nodes.push((format!("AMD GPU {index}"), device.clone()));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                find_amd_gpu_nodes(child, nodes);
            }
        }
        Value::Array(items) => {
            for child in items {
                find_amd_gpu_nodes(child, nodes);
            }
        }
        _ => {}
    }
}

fn parse_amd_smi_json(output: &str) -> Vec<GpuTelemetry> {
    let Ok(root) = serde_json::from_str::<Value>(output) else {
        return Vec::new();
    };
    let mut nodes = Vec::new();
    find_amd_gpu_nodes(&root, &mut nodes);
    if nodes.is_empty()
        && find_json_metric(
            &root,
            &[
                "GFX_ACTIVITY",
                "GPU_ACTIVITY",
                "GPU_USAGE",
                "HOTSPOT",
                "GPU_TEMP",
            ],
        )
        .is_some()
    {
        nodes.push(("AMD GPU".into(), root));
    }
    nodes
        .into_iter()
        .filter_map(|(fallback_name, node)| {
            let utilization = find_json_metric(
                &node,
                &[
                    "GFX_ACTIVITY",
                    "GPU_ACTIVITY",
                    "GPU_USAGE",
                    "GPU_UTILIZATION",
                    "UTILIZATION_GPU",
                ],
            )
            .filter(|value| *value <= 100.0);
            let memory_utilization = find_json_metric(
                &node,
                &["MEM_ACTIVITY", "MEMORY_UTILIZATION", "VRAM_UTILIZATION"],
            )
            .filter(|value| *value <= 100.0);
            let temperature = find_json_metric(
                &node,
                &[
                    "HOTSPOT",
                    "EDGE",
                    "GPU_TEMP",
                    "GPU_TEMPERATURE",
                    "TEMPERATURE_GPU",
                ],
            );
            let power = find_json_metric(
                &node,
                &["SOCKET_POWER", "POWER_USAGE", "POWER_DRAW", "GPU_POWER"],
            );
            let clock = find_json_metric(
                &node,
                &["GFX_CLOCK", "GFXCLK", "CURRENT_GFXCLK", "GFX_FREQUENCY"],
            );
            let name = find_json_text(&node, &["MARKET_NAME", "GPU_NAME", "PRODUCT_NAME", "NAME"])
                .unwrap_or(fallback_name);
            if utilization.is_none()
                && memory_utilization.is_none()
                && temperature.is_none()
                && power.is_none()
                && clock.is_none()
            {
                return None;
            }
            Some(GpuTelemetry {
                name,
                utilization_percent: utilization,
                memory_utilization_percent: memory_utilization,
                memory_total_mib: None,
                memory_used_mib: None,
                temperature_c: temperature,
                graphics_clock_mhz: clock,
                power_draw_watts: power,
                source: "AMD SMI metric JSON (read-only; supported fields only)".into(),
            })
        })
        .collect()
}

fn collect_amd_gpu_telemetry() -> Vec<GpuTelemetry> {
    #[cfg(target_os = "linux")]
    {
        if !executable_exists("amd-smi") {
            return Vec::new();
        }
        let args = vec![
            "metric".into(),
            "--temperature".into(),
            "--usage".into(),
            "--power".into(),
            "--clock".into(),
            "--mem-usage".into(),
            "--json".into(),
        ];
        if let Ok(output) = run_command("amd-smi", &args, 8) {
            if output.exit_code == Some(0) {
                return parse_amd_smi_json(&output.stdout);
            }
        }
    }
    Vec::new()
}

fn collect_numeric_values_for_keys(value: &Value, keys: &[&str], result: &mut Vec<f64>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if keys
                    .iter()
                    .any(|candidate| normalized_json_key(candidate) == normalized_json_key(key))
                {
                    if let Some(number) = metric_value_number(child) {
                        result.push(number);
                    }
                }
                collect_numeric_values_for_keys(child, keys, result);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_numeric_values_for_keys(child, keys, result);
            }
        }
        _ => {}
    }
}

fn parse_intel_gpu_top_json(output: &str) -> Vec<GpuTelemetry> {
    let Ok(root) = serde_json::from_str::<Value>(output) else {
        return Vec::new();
    };
    let mut busy_values = Vec::new();
    collect_numeric_values_for_keys(
        &root,
        &["busy", "busy_percent", "busy_pct"],
        &mut busy_values,
    );
    busy_values.retain(|value| value.is_finite() && (0.0..=100.0).contains(value));
    if busy_values.is_empty() {
        return Vec::new();
    }
    let maximum_busy = busy_values.into_iter().fold(0.0_f64, f64::max);
    let name = find_json_text(&root, &["device_name", "gpu_name", "name"])
        .unwrap_or_else(|| "Intel GPU".into());
    vec![GpuTelemetry {
        name,
        utilization_percent: Some(maximum_busy),
        memory_utilization_percent: None,
        memory_total_mib: None,
        memory_used_mib: None,
        temperature_c: None,
        graphics_clock_mhz: None,
        power_draw_watts: None,
        source: "intel_gpu_top JSON engine busy counters; optional PMU access".into(),
    }]
}

fn collect_intel_gpu_telemetry() -> Vec<GpuTelemetry> {
    #[cfg(target_os = "linux")]
    {
        if !executable_exists("intel_gpu_top") {
            return Vec::new();
        }
        let args = vec![
            "-J".into(),
            "-s".into(),
            "1000".into(),
            "-n".into(),
            "1".into(),
        ];
        if let Ok(output) = run_command("intel_gpu_top", &args, 6) {
            if output.exit_code == Some(0) {
                return parse_intel_gpu_top_json(&output.stdout);
            }
        }
    }
    Vec::new()
}

fn collect_gpu_telemetry() -> Vec<GpuTelemetry> {
    let mut results = collect_nvidia_gpu_telemetry();
    results.extend(collect_amd_gpu_telemetry());
    results.extend(collect_intel_gpu_telemetry());
    results
}

fn vendor_support_profile(manufacturer: &str, model: &str) -> VendorSupportProfile {
    let brand = manufacturer.to_ascii_lowercase();
    let model_lower = model.to_ascii_lowercase();
    let (vendor, support_url, utility, setup, boot, notes, families): (
        &str,
        Option<&str>,
        Option<&str>,
        &str,
        &str,
        &str,
        &[&str],
    ) = if brand.contains("hewlett") || brand == "hp" || brand.starts_with("hp ") {
        (
            "HP",
            Some("https://support.hp.com/"),
            Some("HP PC Hardware Diagnostics UEFI / HP Support Assistant"),
            "Esc, then F10 on many models",
            "Esc, then F9 on many models",
            "Verify the exact product number and BIOS instructions before firmware work.",
            &[
                "elitebook",
                "probook",
                "zbook",
                "spectre",
                "envy",
                "pavilion",
                "omen",
                "victus",
            ],
        )
    } else if brand.contains("dell") || brand.contains("alienware") {
        ("Dell", Some("https://www.dell.com/support/home/"), Some("Dell SupportAssist / ePSA pre-boot diagnostics"), "F2 on many models", "F12 on many models", "Use the Service Tag only on Dell's official support site; pre-boot tests vary by model.", &["latitude", "inspiron", "optiplex", "precision", "xps", "vostro", "alienware"])
    } else if brand.contains("lenovo") {
        (
            "Lenovo",
            Some("https://support.lenovo.com/"),
            Some("Lenovo Diagnostics / Lenovo Vantage"),
            "F1 or F2; Novo button on supported models",
            "F12 or Novo button on supported models",
            "ThinkPad, ThinkBook, IdeaPad and Legion entry methods differ by model generation.",
            &[
                "thinkpad",
                "thinkcentre",
                "thinkbook",
                "ideapad",
                "yoga",
                "legion",
                "loq",
            ],
        )
    } else if brand.contains("acer") {
        (
            "Acer",
            Some("https://www.acer.com/support"),
            Some("Acer Diagnostics / Care Center where supported"),
            "F2 on many models",
            "F12 on many models; may need enabling in firmware",
            "Confirm the exact model manual before changing boot configuration.",
            &["aspire", "swift", "spin", "travelmate", "predator", "nitro"],
        )
    } else if brand.contains("asus") || brand.contains("asustek") {
        (
            "ASUS",
            Some("https://www.asus.com/support/"),
            Some("MyASUS / ASUS System Diagnosis where supported"),
            "F2 or Delete on many models",
            "Esc on many models",
            "Use only drivers and firmware matching the complete product model.",
            &["vivobook", "zenbook", "expertbook", "proart", "rog", "tuf"],
        )
    } else if brand.contains("micro-star") || brand == "msi" || brand.contains("msi computer") {
        (
            "MSI",
            Some("https://www.msi.com/support"),
            Some("MSI Center / MSI pre-boot diagnostics where supported"),
            "Delete on many models",
            "F11 on many models",
            "Firmware keys and available tests vary between MSI laptop and motherboard families.",
            &[
                "katana", "raider", "titan", "stealth", "prestige", "modern", "cyborg",
            ],
        )
    } else if brand.contains("microsoft") || brand.contains("surface") {
        (
            "Microsoft Surface",
            Some("https://support.microsoft.com/surface"),
            Some("Surface Diagnostic Toolkit where supported"),
            "Hold Volume Up while pressing Power on supported Surface devices",
            "Use the documented Surface boot sequence",
            "Follow the model-specific Surface UEFI documentation.",
            &[
                "surface pro",
                "surface laptop",
                "surface go",
                "surface book",
            ],
        )
    } else if brand.contains("samsung") {
        (
            "Samsung",
            Some("https://www.samsung.com/support/"),
            Some("Samsung Update / Samsung Settings where supported"),
            "F2 on many models",
            "F10 or Esc on some models",
            "Check the model-specific Samsung support manual.",
            &["galaxy book"],
        )
    } else if brand.contains("gigabyte") {
        (
            "GIGABYTE",
            Some("https://www.gigabyte.com/Support"),
            Some("GIGABYTE Control Center / UEFI diagnostics where supported"),
            "Delete on many systems",
            "F12 on many systems",
            "Motherboard and laptop firmware procedures differ.",
            &["aero", "aorus"],
        )
    } else if brand.contains("apple") {
        ("Apple (Windows/Boot Camp or supported hardware context)", Some("https://support.apple.com/"), Some("Apple Diagnostics runs in macOS recovery environment"), "Use the model-specific Apple startup procedure", "Use the model-specific startup options", "CrossDiag does not replace Apple Diagnostics or support firmware modification on Apple hardware.", &["macbook", "imac", "mac mini", "mac pro"])
    } else if brand.contains("huawei") {
        (
            "HUAWEI",
            Some("https://consumer.huawei.com/support/"),
            Some("HUAWEI PC Manager where supported"),
            "F2 on many models",
            "F12 on some models",
            "Check the exact MateBook model instructions.",
            &["matebook"],
        )
    } else if brand.contains("dynabook") || brand.contains("toshiba") {
        (
            "Dynabook / Toshiba",
            Some("https://www.dynabook.com/support"),
            Some("Dynabook PC Health Monitor where supported"),
            "F2 on many models",
            "F12 on many models",
            "Legacy Toshiba and newer Dynabook models have different support resources.",
            &["satellite", "portege", "tecra"],
        )
    } else if brand.contains("xiaomi") || brand.contains("redmi") {
        (
            "Xiaomi / Redmi",
            Some("https://www.mi.com/global/support/"),
            Some("Xiaomi PC Manager where supported"),
            "F2 on many models",
            "F12 on some models",
            "Use the exact regional model and official support package.",
            &["redmibook", "xiaomi book", "mi notebook"],
        )
    } else if brand.contains("honor") {
        (
            "HONOR",
            Some("https://www.honor.com/global/support/"),
            Some("HONOR PC Manager where supported"),
            "F2 on many models",
            "F12 on some models",
            "Confirm the product's regional model identifier before downloading drivers.",
            &["magicbook"],
        )
    } else if brand.contains("fujitsu") {
        (
            "Fujitsu",
            Some("https://support.ts.fujitsu.com/"),
            Some("Fujitsu Diagnostics / DeskUpdate where supported"),
            "F2 on many models",
            "F12 or Esc on some models",
            "Business and workstation models can have model-specific service procedures.",
            &["lifebook", "esprimo", "celius", "celsius"],
        )
    } else if brand.contains("panasonic") {
        (
            "Panasonic",
            Some("https://pc-dl.panasonic.co.jp/itn/support/"),
            Some("Panasonic PC Settings / Toughbook diagnostics where supported"),
            "F2 or F9 on some models",
            "F12 or F2 on some models",
            "Rugged Toughbook hardware uses model-specific diagnostics and firmware guidance.",
            &["toughbook", "toughpad"],
        )
    } else if brand.contains("razer") {
        (
            "Razer",
            Some("https://mysupport.razer.com/"),
            Some("Razer Synapse and model-specific support diagnostics"),
            "F1, F2 or Delete depending on model",
            "F12 or Esc depending on model",
            "Check the exact Blade model and region before using firmware or drivers.",
            &["blade", "book"],
        )
    } else if brand.contains("lg electronics") || brand == "lg" {
        (
            "LG",
            Some("https://www.lg.com/us/support/"),
            Some("LG Update Center / LG Smart Assistant where supported"),
            "F2 on many models",
            "F10 or F12 on some models",
            "LG Gram support and firmware downloads are model- and region-specific.",
            &["gram", "ultrapc"],
        )
    } else if brand.contains("vaio") || brand.contains("sony") {
        (
            "VAIO / Sony",
            Some("https://support.us.vaio.com/"),
            Some("VAIO Care / VAIO Update where supported"),
            "Assist button, F2 or model-specific key",
            "Assist button or F11/F12 depending on model",
            "Legacy Sony VAIO and current VAIO product support may use different regional portals.",
            &["vaio", "sx"],
        )
    } else if brand.contains("medion") {
        (
            "MEDION",
            Some("https://www.medion.com/gb/service/"),
            Some("MEDION Service App / model-specific diagnostics"),
            "F2 on many models",
            "F10 or F12 on some models",
            "Use the full MSN/product number for the correct drivers and manuals.",
            &["akoya", "erazer"],
        )
    } else if brand.contains("framework") {
        ("Framework", Some("https://frame.work/support"), Some("Framework support guides and hardware diagnostics"), "F2 or F12 on many models", "F12 on many models", "Use Framework's model-specific guides; module replacement procedures differ by generation.", &["laptop"])
    } else if brand.contains("system76") {
        (
            "System76",
            Some("https://support.system76.com/"),
            Some("System76 Diagnostics / firmware manager where supported"),
            "F2 or Delete depending on model",
            "F7/F10/F12 depending on model",
            "System76 firmware and Pop!_OS guidance is model-specific.",
            &["gazelle", "lemur", "oryx", "pangolin", "thelio"],
        )
    } else if brand.contains("asrock") {
        (
            "ASRock",
            Some("https://www.asrock.com/support/"),
            Some("UEFI diagnostics and vendor motherboard utilities"),
            "F2 or Delete on many boards",
            "F11 on many boards",
            "Use the exact motherboard model and revision for BIOS and driver packages.",
            &["b650", "b550", "x670", "z790", "steel legend", "taichi"],
        )
    } else {
        ("Unknown / custom manufacturer", None, None, "Check the exact model manual; common keys include F2 or Delete", "Check the exact model manual; common keys include F12 or Esc", "No manufacturer was confidently matched. Do not use another vendor's firmware package.", &[])
    };
    let product_family = families
        .iter()
        .find(|family| model_lower.contains(**family))
        .map(|family| family.to_string());
    VendorSupportProfile {
        vendor: vendor.into(),
        product_family,
        support_url: support_url.map(str::to_string),
        diagnostic_utility: utility.map(str::to_string),
        firmware_setup_hint: setup.into(),
        boot_menu_hint: boot.into(),
        notes: notes.into(),
    }
}

fn pnp_problem_description(code: u32) -> &'static str {
    match code {
        1 => "The device is not configured correctly.",
        3 => "The driver may be corrupted, or system resources may be insufficient.",
        10 => "The device cannot start.",
        12 => "The device cannot obtain sufficient free resources.",
        14 => "The device requires a system restart.",
        18 => "Windows reports that the device drivers must be reinstalled.",
        19 => "The device configuration in the registry is damaged or incomplete.",
        22 => "The device is disabled.",
        24 => "The device may be absent, malfunctioning or incompletely configured.",
        28 => "A driver is not installed for this device.",
        31 => "Windows cannot load the required driver.",
        32 => "The device driver service is disabled.",
        37 => "The device driver failed initialization.",
        39 => "The driver may be missing or corrupted.",
        43 => "Windows stopped the device because it reported a problem.",
        45 => "Windows currently reports the device as not connected.",
        46 => "The device is unavailable while the operating system is shutting down.",
        48 => "Windows blocked the driver because of a known compatibility problem.",
        52 => "Windows could not verify the driver's digital signature.",
        _ => "Windows reported a Configuration Manager problem code; inspect the full Device Manager details.",
    }
}

fn collect_driver_findings(
    drivers: &[DriverInfo],
    problems: &[ProblemDevice],
) -> Vec<DriverFinding> {
    let mut findings = Vec::new();
    for device in problems {
        let explanation = match device.problem_code {
            Some(code) => format!(
                "Windows Configuration Manager Code {code}: {} OS status: {}.",
                pnp_problem_description(code),
                device.status
            ),
            None => format!(
                "The operating system reports this device as {}.",
                device.status
            ),
        };
        let recommendation = match device.problem_code {
            Some(22) => "Confirm that the device is intended to be enabled, then review it in Device Manager. Do not enable unknown hardware without understanding its purpose.".to_string(),
            Some(28) => "Identify the exact hardware ID and obtain a compatible driver from Windows Update or the computer/device manufacturer's official support page.".to_string(),
            Some(43) => "Check device event details, cable/power conditions and recent driver changes. Code 43 does not identify one specific failed part.".to_string(),
            Some(14) => "Save work and perform a normal operating-system restart, then scan the device again.".to_string(),
            _ => "Review the full Device Manager status, device hardware ID, recent driver changes and official manufacturer support before changing drivers.".to_string(),
        };
        let code = device
            .problem_code
            .map(|value| format!("PNP_CONFIG_MANAGER_{value}"))
            .unwrap_or_else(|| "OS_DEVICE_PROBLEM".into());
        findings.push(DriverFinding {
            device: device.name.clone(),
            severity: "warning".into(),
            code,
            explanation,
            recommendation,
        });
    }
    for driver in drivers {
        if driver.source == "lspci -nnk"
            && driver
                .driver
                .as_deref()
                .is_some_and(|value| value.starts_with("Not bound; candidate kernel modules:"))
        {
            findings.push(DriverFinding {
                device: driver.device.clone(), severity: "info".into(), code: "LINUX_DRIVER_NOT_BOUND_CANDIDATE_MODULES".into(),
                explanation: "PCI inventory lists candidate kernel modules, but no kernel driver is currently bound to this device.".into(),
                recommendation: "Confirm whether this device should be active on this computer. Check the distribution's hardware documentation and kernel logs before loading or installing a driver.".into(),
            });
        }
        if driver.signed == Some(false) {
            findings.push(DriverFinding {
                device: driver.device.clone(), severity: "warning".into(), code: "DRIVER_SIGNATURE_NOT_CONFIRMED".into(),
                explanation: "The Windows driver inventory reports that this driver is not signed.".into(),
                recommendation: "Confirm whether the driver is expected and obtain a signed replacement from Windows Update or the exact device manufacturer's official site.".into(),
            });
        }
        if driver.version.as_deref().unwrap_or("").trim().is_empty()
            && driver.source == "Win32_PnPSignedDriver"
        {
            findings.push(DriverFinding {
                device: driver.device.clone(), severity: "info".into(), code: "DRIVER_VERSION_UNAVAILABLE".into(),
                explanation: "The operating system did not expose a driver version for this record.".into(),
                recommendation: "Check the device's properties and official support page. A missing version field does not prove that the driver is outdated.".into(),
            });
        }
    }
    findings
}

#[cfg(target_os = "windows")]
fn collect_windows_hardware() -> Result<Value, String> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [Console]::OutputEncoding
$ErrorActionPreference = 'SilentlyContinue'
$gpus = @(Get-CimInstance Win32_VideoController | ForEach-Object { [pscustomobject]@{ name=$_.Name; driverVersion=$_.DriverVersion; status=$_.Status; adapterRamBytes=$_.AdapterRAM } })
$memoryModules = @(Get-CimInstance Win32_PhysicalMemory | ForEach-Object { [pscustomobject]@{ locator=$_.DeviceLocator; manufacturer=$_.Manufacturer; partNumber=$_.PartNumber; capacityBytes=$_.Capacity; speedMhz=$_.Speed; configuredSpeedMhz=$_.ConfiguredClockSpeed } })
$drivers = @(Get-CimInstance Win32_PnPSignedDriver | Select-Object -First 150 | ForEach-Object { [pscustomobject]@{ device=$_.DeviceName; manufacturer=$_.Manufacturer; driver=$_.DriverName; version=$_.DriverVersion; signed=$_.IsSigned } })
$peripherals = @(Get-PnpDevice | Where-Object { $_.Class -in @('USB','Monitor','Bluetooth','Media','Camera','Printer','Image') } | Select-Object -First 120 | ForEach-Object { [pscustomobject]@{ category=$_.Class; name=$_.FriendlyName; status=$_.Status; instanceId=$_.InstanceId } })
$batteries = @(Get-CimInstance Win32_Battery | ForEach-Object { [pscustomobject]@{ name=$_.Name; status=[string]$_.BatteryStatus; chargePercent=$_.EstimatedChargeRemaining; designCapacity=$null; fullChargeCapacity=$null } })
# ACPI battery wear counters are not exposed on every model. Only pair them when exactly
# one Windows battery and one record of each WMI class exist; otherwise leave wear unknown.
if ($batteries.Count -eq 1) {
  $staticCapacity = @(Get-CimInstance -Namespace root/wmi -ClassName BatteryStaticData | Where-Object { $_.DesignedCapacity -gt 0 })
  $fullCapacity = @(Get-CimInstance -Namespace root/wmi -ClassName BatteryFullChargedCapacity | Where-Object { $_.FullChargedCapacity -gt 0 })
  if ($staticCapacity.Count -eq 1 -and $fullCapacity.Count -eq 1) {
    $batteries[0].designCapacity = [uint64]$staticCapacity[0].DesignedCapacity
    $batteries[0].fullChargeCapacity = [uint64]$fullCapacity[0].FullChargedCapacity
  }
}
$thermalSensors = @(Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature | Where-Object { $_.CurrentTemperature -gt 0 } | ForEach-Object {
  $celsius = ($_.CurrentTemperature / 10) - 273.15
  $critical = if ($_.CriticalTripPoint -gt 0) { ($_.CriticalTripPoint / 10) - 273.15 } else { $null }
  [pscustomobject]@{ name=$_.InstanceName; category='temperature'; value=[math]::Round($celsius,1); unit='°C'; status='ACPI thermal-zone reading'; criticalLimit=$critical; source='MSAcpi_ThermalZoneTemperature' }
})
$fanSensors = @(Get-CimInstance Win32_Fan | Where-Object { $null -ne $_.DesiredSpeed } | ForEach-Object {
  [pscustomobject]@{ name=($_.Name -as [string]); category='fan_target'; value=[double]$_.DesiredSpeed; unit='RPM requested'; status='target speed reported; actual RPM not verified'; criticalLimit=$null; source='Win32_Fan.DesiredSpeed' }
})
$sensors = @($thermalSensors) + @($fanSensors)
$problems = @(Get-CimInstance Win32_PnPEntity | Where-Object { [uint32]$_.ConfigManagerErrorCode -gt 0 } | Select-Object -First 60 | ForEach-Object {
  $problemCode = [uint32]$_.ConfigManagerErrorCode
  [pscustomobject]@{ name=$_.Name; class=$_.PNPClass; status=("Configuration Manager error {0}" -f $problemCode); problemCode=$problemCode; instanceId=$_.PNPDeviceID }
})
$networks = @(Get-NetAdapter | ForEach-Object { $adapter = $_; $stats = Get-NetAdapterStatistics -Name $adapter.Name; [pscustomobject]@{ name=$adapter.Name; status=[string]$adapter.Status; linkSpeed=[string]$adapter.LinkSpeed; macAddress=$adapter.MacAddress; receivedBytes=$stats.ReceivedBytes; transmittedBytes=$stats.SentBytes } })
$bios = Get-CimInstance Win32_BIOS
$secureBoot = $null
try { $secureBoot = [bool](Confirm-SecureBootUEFI -ErrorAction Stop) } catch { }
$firmwareMode = $null
try { $firmwareMode = [string](Get-ComputerInfo -Property BiosFirmwareType).BiosFirmwareType } catch { }
$tpm = $null
try { $tpm = [bool](Get-Tpm -ErrorAction Stop).TpmPresent } catch { }
[pscustomobject]@{
  gpus=@($gpus); memoryModules=@($memoryModules); drivers=@($drivers); peripherals=@($peripherals); batteries=@($batteries); problems=@($problems); networks=@($networks); sensors=@($sensors)
  biosVendor=$bios.Manufacturer; biosVersion=$bios.SMBIOSBIOSVersion; biosReleaseDate=[string]$bios.ReleaseDate
  firmwareMode=$firmwareMode; secureBoot=$secureBoot; tpmDetected=$tpm
} | ConvertTo-Json -Depth 5 -Compress
"#;
    let output = run_command(
        "powershell.exe",
        &[
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            script.into(),
        ],
        35,
    )?;
    serde_json::from_str::<Value>(&output.stdout)
        .map_err(|error| format!("Windows hardware inventory returned invalid JSON: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn collect_windows_hardware() -> Result<Value, String> {
    Ok(Value::Null)
}

fn collect_linux_memory_modules() -> Vec<MemoryModule> {
    let mut modules = Vec::new();
    let base = std::path::Path::new("/sys/devices/system/edac/mc");
    let Ok(controllers) = fs::read_dir(base) else {
        return modules;
    };
    for controller in controllers.flatten() {
        let Ok(dimms) = fs::read_dir(controller.path()) else {
            continue;
        };
        for dimm in dimms.flatten() {
            let path = dimm.path();
            let size_mib = read_number(&path.join("size"));
            if size_mib.unwrap_or(0) == 0 {
                continue;
            }
            modules.push(MemoryModule {
                locator: read_text(&path.join("dimm_label"))
                    .or_else(|| Some(dimm.file_name().to_string_lossy().to_string())),
                manufacturer: read_text(&path.join("dimm_manufacturer")),
                part_number: read_text(&path.join("dimm_part_number")),
                capacity_bytes: size_mib.map(|value| value.saturating_mul(1024 * 1024)),
                speed_mhz: None,
                configured_speed_mhz: None,
                source: path.to_string_lossy().to_string(),
            });
        }
    }
    modules
}

fn parse_lspci_driver_output(output: &str) -> Vec<DriverInfo> {
    let mut drivers = Vec::new();
    let mut current_device: Option<String> = None;
    let mut in_use: Option<String> = None;
    let mut modules: Option<String> = None;
    let flush = |device: Option<String>,
                 driver: Option<String>,
                 modules: Option<String>,
                 list: &mut Vec<DriverInfo>| {
        if let Some(device_name) = device {
            if driver.is_some() || modules.is_some() {
                let driver_value = driver.or_else(|| {
                    modules
                        .as_ref()
                        .map(|value| format!("Not bound; candidate kernel modules: {value}"))
                });
                list.push(DriverInfo {
                    device: device_name,
                    manufacturer: None,
                    driver: driver_value,
                    version: None,
                    signed: None,
                    source: "lspci -nnk".into(),
                });
            }
        }
    };
    for line in output.lines() {
        if !line.starts_with('\t') && !line.starts_with(' ') && !line.trim().is_empty() {
            flush(
                current_device.take(),
                in_use.take(),
                modules.take(),
                &mut drivers,
            );
            current_device = Some(line.trim().to_string());
        } else {
            let item = line.trim();
            if let Some(value) = item.strip_prefix("Kernel driver in use:") {
                in_use = Some(value.trim().to_string());
            } else if let Some(value) = item.strip_prefix("Kernel modules:") {
                modules = Some(value.trim().to_string());
            }
        }
    }
    flush(current_device, in_use, modules, &mut drivers);
    drivers
}

fn collect_linux_drivers() -> Vec<DriverInfo> {
    if !executable_exists("lspci") {
        return Vec::new();
    }
    let Ok(output) = run_command("lspci", &["-nnk".into()], 15) else {
        return Vec::new();
    };
    parse_lspci_driver_output(&output.stdout)
}

fn collect_linux_peripherals() -> Vec<PeripheralInfo> {
    let mut peripherals = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/bus/usb/devices") {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.join("idVendor").exists() || entry.file_name().to_string_lossy().contains(':')
            {
                continue;
            }
            let vendor = read_text(&path.join("idVendor"));
            let product_id = read_text(&path.join("idProduct"));
            let name = read_text(&path.join("product")).unwrap_or_else(|| "USB device".into());
            let identifier = match (vendor, product_id) {
                (Some(v), Some(p)) => Some(format!("{v}:{p}")),
                _ => None,
            };
            peripherals.push(PeripheralInfo {
                category: "USB".into(),
                name,
                status: Some("enumerated by kernel".into()),
                identifier,
                link_speed: read_text(&path.join("speed")).map(|value| format!("{value} Mb/s")),
                source: path.to_string_lossy().to_string(),
            });
        }
    }
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("card") || !name.contains('-') {
                continue;
            }
            let status = read_text(&entry.path().join("status"));
            let modes = read_text(&entry.path().join("modes"));
            peripherals.push(PeripheralInfo {
                category: "Display connector".into(),
                name: name.clone(),
                status: status.clone(),
                identifier: modes,
                link_speed: None,
                source: entry.path().to_string_lossy().to_string(),
            });
        }
    }
    peripherals
}

fn collect_linux_gpus() -> Vec<GpuDevice> {
    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return result;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card")
            || name.contains('-')
            || !name[4..].chars().all(|ch| ch.is_ascii_digit())
        {
            continue;
        }
        let device_path = entry.path().join("device");
        if !device_path.exists() {
            continue;
        }
        let vendor = read_text(&device_path.join("vendor"));
        let device = read_text(&device_path.join("device"));
        let driver = fs::read_link(device_path.join("driver"))
            .ok()
            .and_then(|path| {
                path.file_name()
                    .map(|value| value.to_string_lossy().to_string())
            });
        let vram = read_number(&device_path.join("mem_info_vram_total"));
        let label = match (&vendor, &device) {
            (Some(v), Some(d)) => format!("Display controller {v}:{d}"),
            _ => format!("Display controller {name}"),
        };
        result.push(GpuDevice {
            name: label,
            vendor_id: vendor,
            device_id: device,
            driver,
            status: "enumerated; functional test not performed".into(),
            vram_bytes: vram,
            source: "/sys/class/drm".into(),
        });
    }
    result
}

fn collect_linux_power_supplies() -> Vec<BatteryInfo> {
    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/power_supply") else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let kind = read_text(&path.join("type")).unwrap_or_default();
        if !kind.eq_ignore_ascii_case("battery") {
            continue;
        }
        let name = read_text(&path.join("model_name"))
            .or_else(|| Some(entry.file_name().to_string_lossy().to_string()))
            .unwrap_or_else(|| "Battery".into());
        let charge = read_number(&path.join("capacity")).map(|value| value.min(100) as f64);
        let full_energy = read_number(&path.join("energy_full"));
        let design_energy = read_number(&path.join("energy_full_design"));
        let full_charge = read_number(&path.join("charge_full"));
        let design_charge = read_number(&path.join("charge_full_design"));
        let (full_for_health, design_for_health) = match (full_energy, design_energy) {
            (Some(full), Some(design)) => (Some(full), Some(design)),
            _ => (full_charge, design_charge),
        };
        let health = calculate_battery_health(full_for_health, design_for_health);
        result.push(BatteryInfo {
            name,
            status: read_text(&path.join("status")),
            charge_percent: charge,
            health_percent: health,
            // Only energy_* sysfs counters use micro-watt-hours and may be labelled mWh.
            // charge_* counters use micro-amp-hours, so do not mislabel them as energy.
            design_capacity_mwh: design_energy.map(|value| value / 1000),
            full_charge_capacity_mwh: full_energy.map(|value| value / 1000),
            source: path.to_string_lossy().to_string(),
            limitations: if health.is_none() { vec!["Battery wear could not be calculated because full and design capacity were not both exposed.".into()] } else { Vec::new() },
        });
    }
    result
}

fn collect_linux_sensors() -> Vec<HardwareSensor> {
    let mut sensors = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return sensors;
    };
    for entry in entries.flatten() {
        let base = entry.path();
        let chip = read_text(&base.join("name"))
            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
        let Ok(files) = fs::read_dir(&base) else {
            continue;
        };
        for file in files.flatten() {
            let file_name = file.file_name().to_string_lossy().to_string();
            if let Some(index) = file_name
                .strip_prefix("temp")
                .and_then(|s| s.strip_suffix("_input"))
            {
                if let Some(raw) = read_number(&file.path()) {
                    let label = read_text(&base.join(format!("temp{index}_label")))
                        .unwrap_or_else(|| format!("Temperature {index}"));
                    let critical = read_number(&base.join(format!("temp{index}_crit")))
                        .map(|v| v as f64 / 1000.0);
                    let value = raw as f64 / 1000.0;
                    let status = if critical.is_some_and(|limit| value >= limit) {
                        "critical"
                    } else {
                        "measured"
                    };
                    sensors.push(HardwareSensor {
                        name: format!("{chip}: {label}"),
                        category: "temperature".into(),
                        value,
                        unit: "°C".into(),
                        status: status.into(),
                        critical_limit: critical,
                        source: file.path().to_string_lossy().to_string(),
                    });
                }
            } else if let Some(index) = file_name
                .strip_prefix("fan")
                .and_then(|s| s.strip_suffix("_input"))
            {
                if let Some(raw) = read_number(&file.path()) {
                    let label = read_text(&base.join(format!("fan{index}_label")))
                        .unwrap_or_else(|| format!("Fan {index}"));
                    sensors.push(HardwareSensor {
                        name: format!("{chip}: {label}"),
                        category: "fan".into(),
                        value: raw as f64,
                        unit: "RPM".into(),
                        status: if raw == 0 {
                            "zero RPM reported; fan may be stopped or sensor unavailable".into()
                        } else {
                            "measured".into()
                        },
                        critical_limit: None,
                        source: file.path().to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    sensors
}

fn read_linux_secure_boot() -> Option<bool> {
    let path = std::path::Path::new(
        "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c",
    );
    let bytes = fs::read(path).ok()?;
    // efivarfs prepends four attribute bytes; the variable payload follows them.
    bytes.get(4).map(|value| *value == 1)
}

fn collect_linux_firmware() -> FirmwareInfo {
    let dmi = std::path::Path::new("/sys/class/dmi/id");
    FirmwareInfo {
        vendor: read_text(&dmi.join("bios_vendor")),
        version: read_text(&dmi.join("bios_version")),
        release_date: read_text(&dmi.join("bios_date")),
        mode: if std::path::Path::new("/sys/firmware/efi").exists() {
            Some("UEFI".into())
        } else {
            Some("Legacy BIOS or EFI information unavailable".into())
        },
        secure_boot: read_linux_secure_boot(),
        tpm_detected: fs::read_dir("/sys/class/tpm")
            .ok()
            .map(|mut entries| entries.next().is_some()),
        source: "/sys/class/dmi/id and /sys/firmware".into(),
    }
}

fn collect_hardware_diagnostics_blocking() -> Result<HardwareDiagnostics, String> {
    let mut system = System::new_all();
    system.refresh_all();
    // CPU usage is calculated from two samples; wait for a short measurement window.
    thread::sleep(Duration::from_millis(250));
    system.refresh_cpu_usage();
    let cpu_usage = (system.global_cpu_usage() as f64).clamp(0.0, 100.0);
    let cpu_frequency = system
        .cpus()
        .iter()
        .map(|cpu| cpu.frequency())
        .max()
        .filter(|value| *value > 0);
    let cpu_cores = system
        .cpus()
        .iter()
        .map(|cpu| CpuCoreInfo {
            name: if cpu.name().trim().is_empty() {
                "CPU core".into()
            } else {
                cpu.name().to_string()
            },
            usage_percent: (cpu.cpu_usage() as f64).clamp(0.0, 100.0),
            frequency_mhz: (cpu.frequency() > 0).then_some(cpu.frequency()),
        })
        .collect::<Vec<_>>();
    let gpu_telemetry = collect_gpu_telemetry();
    let (detected_manufacturer, detected_model) = platform_computer_identity();
    let vendor_support = vendor_support_profile(&detected_manufacturer, &detected_model);
    let mut sensors = Vec::new();
    let components = sysinfo::Components::new_with_refreshed_list();
    for component in &components {
        if let Some(temp) = component.temperature().filter(|value| value.is_finite()) {
            let critical_limit = component.critical().map(|value| value as f64);
            let status = if critical_limit.is_some_and(|limit| (temp as f64) >= limit) {
                "critical"
            } else {
                "measured; platform sensor threshold may be unavailable"
            };
            sensors.push(HardwareSensor {
                name: component.label().to_string(),
                category: "temperature".into(),
                value: temp as f64,
                unit: "°C".into(),
                status: status.into(),
                critical_limit,
                source: "sysinfo hardware components".into(),
            });
        }
    }

    let mut memory_modules = Vec::new();
    let mut drivers = Vec::new();
    let mut peripherals = Vec::new();
    let mut gpus = Vec::new();
    let mut batteries = Vec::new();
    let mut problem_devices = Vec::new();
    let mut network_adapters = Vec::new();
    let mut firmware = collect_linux_firmware();
    let mut limitations = vec![
        "A device appearing in inventory does not prove that it passes a functional stress test.".into(),
        "Physical connector condition, intermittent faults and PSU output require external tests or inspection.".into(),
        "CPU and GPU load/stress tests are deliberately not started by this inventory scan.".into(),
    ];

    #[cfg(target_os = "windows")]
    {
        match collect_windows_hardware() {
            Ok(data) => {
                for item in json_array(data.get("memoryModules")) {
                    memory_modules.push(MemoryModule {
                        locator: json_string(item.get("locator")),
                        manufacturer: json_string(item.get("manufacturer")),
                        part_number: json_string(item.get("partNumber")),
                        capacity_bytes: json_number(item.get("capacityBytes")),
                        speed_mhz: json_number(item.get("speedMhz")),
                        configured_speed_mhz: json_number(item.get("configuredSpeedMhz")),
                        source: "Win32_PhysicalMemory".into(),
                    });
                }
                for item in json_array(data.get("drivers")) {
                    drivers.push(DriverInfo {
                        device: json_string(item.get("device"))
                            .unwrap_or_else(|| "Unnamed device".into()),
                        manufacturer: json_string(item.get("manufacturer")),
                        driver: json_string(item.get("driver")),
                        version: json_string(item.get("version")),
                        signed: item.get("signed").and_then(Value::as_bool),
                        source: "Win32_PnPSignedDriver".into(),
                    });
                }
                for item in json_array(data.get("peripherals")) {
                    peripherals.push(PeripheralInfo {
                        category: json_string(item.get("category"))
                            .unwrap_or_else(|| "Peripheral".into()),
                        name: json_string(item.get("name"))
                            .unwrap_or_else(|| "Unnamed peripheral".into()),
                        status: json_string(item.get("status")),
                        identifier: json_string(item.get("instanceId")),
                        link_speed: None,
                        source: "Get-PnpDevice".into(),
                    });
                }
                for item in json_array(data.get("gpus")) {
                    gpus.push(GpuDevice {
                        name: json_string(item.get("name"))
                            .unwrap_or_else(|| "Graphics adapter".into()),
                        vendor_id: None,
                        device_id: None,
                        driver: json_string(item.get("driverVersion")),
                        status: json_string(item.get("status"))
                            .unwrap_or_else(|| "not reported".into()),
                        vram_bytes: json_number(item.get("adapterRamBytes"))
                            .filter(|bytes| *bytes < u32::MAX as u64),
                        source: "Win32_VideoController".into(),
                    });
                }
                for item in json_array(data.get("batteries")) {
                    batteries.push(BatteryInfo {
                        name: json_string(item.get("name")).unwrap_or_else(|| "Battery".into()),
                        status: json_string(item.get("status")), charge_percent: json_float(item.get("chargePercent")),
                        health_percent: calculate_battery_health(json_number(item.get("fullChargeCapacity")), json_number(item.get("designCapacity"))),
                        design_capacity_mwh: json_number(item.get("designCapacity")), full_charge_capacity_mwh: json_number(item.get("fullChargeCapacity")),
                        source: "Win32_Battery and ACPI WMI capacity counters when uniquely available".into(),
                        limitations: if calculate_battery_health(json_number(item.get("fullChargeCapacity")), json_number(item.get("designCapacity"))).is_none() { vec!["Battery wear could not be calculated: Windows did not expose a safe, unambiguous design/full-charge capacity pair.".into()] } else { Vec::new() },
                    });
                }
                for item in json_array(data.get("problems")) {
                    problem_devices.push(ProblemDevice {
                        name: json_string(item.get("name"))
                            .unwrap_or_else(|| "Unnamed device".into()),
                        class: json_string(item.get("class")),
                        status: json_string(item.get("status")).unwrap_or_else(|| "unknown".into()),
                        instance_id: json_string(item.get("instanceId")),
                        problem_code: json_number(item.get("problemCode"))
                            .map(|value| value as u32),
                        problem_description: json_number(item.get("problemCode"))
                            .map(|value| pnp_problem_description(value as u32).to_string()),
                        source: "Win32_PnPEntity".into(),
                    });
                }
                for item in json_array(data.get("networks")) {
                    network_adapters.push(NetworkAdapterInfo {
                        name: json_string(item.get("name"))
                            .unwrap_or_else(|| "Network adapter".into()),
                        status: json_string(item.get("status")),
                        link_speed: json_string(item.get("linkSpeed")),
                        mac_address: json_string(item.get("macAddress")),
                        received_bytes: json_number(item.get("receivedBytes")).unwrap_or(0),
                        transmitted_bytes: json_number(item.get("transmittedBytes")).unwrap_or(0),
                        source: "Get-NetAdapter".into(),
                    });
                }
                for item in json_array(data.get("sensors")) {
                    if let Some(value) = json_float(item.get("value")) {
                        let category =
                            json_string(item.get("category")).unwrap_or_else(|| "sensor".into());
                        let critical_limit = json_float(item.get("criticalLimit"));
                        let reported_status =
                            json_string(item.get("status")).unwrap_or_else(|| "measured".into());
                        let status = if category == "temperature"
                            && critical_limit.is_some_and(|limit| value >= limit)
                        {
                            "critical".to_string()
                        } else {
                            reported_status
                        };
                        sensors.push(HardwareSensor {
                            name: json_string(item.get("name"))
                                .unwrap_or_else(|| "Unnamed sensor".into()),
                            category,
                            value,
                            unit: json_string(item.get("unit")).unwrap_or_else(|| "unknown".into()),
                            status,
                            critical_limit,
                            source: json_string(item.get("source"))
                                .unwrap_or_else(|| "Windows hardware sensor interface".into()),
                        });
                    }
                }
                firmware = FirmwareInfo {
                    vendor: json_string(data.get("biosVendor")),
                    version: json_string(data.get("biosVersion")),
                    release_date: json_string(data.get("biosReleaseDate")),
                    mode: json_string(data.get("firmwareMode")),
                    secure_boot: data.get("secureBoot").and_then(Value::as_bool),
                    tpm_detected: data.get("tpmDetected").and_then(Value::as_bool),
                    source: "Win32_BIOS, Get-ComputerInfo, Confirm-SecureBootUEFI and Get-Tpm"
                        .into(),
                };
            }
            Err(error) => limitations.push(format!(
                "Windows-specific inventory could not be collected: {error}"
            )),
        }
        limitations.push("Windows Win32_VideoController AdapterRAM is a limited inventory field and can under-report modern GPUs with more than 4 GiB of VRAM; missing values are not treated as zero.".into());
        if sensors.is_empty() {
            limitations.push("No hardware temperature or fan reading was exposed by the available generic, ACPI or WMI sensor interfaces. A vendor sensor library may expose additional readings.".into());
        }
    }

    #[cfg(target_os = "linux")]
    {
        memory_modules = collect_linux_memory_modules();
        drivers = collect_linux_drivers();
        peripherals = collect_linux_peripherals();
        gpus = collect_linux_gpus();
        batteries = collect_linux_power_supplies();
        let hwmon = collect_linux_sensors();
        for sensor in hwmon {
            // Prefer detailed hwmon values; generic sysinfo may report the same chip as well.
            if !sensors
                .iter()
                .any(|current| current.name.eq_ignore_ascii_case(&sensor.name))
            {
                sensors.push(sensor);
            }
        }
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            let networks = sysinfo::Networks::new_with_refreshed_list();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "lo" {
                    continue;
                }
                let stats = networks.get(&name);
                network_adapters.push(NetworkAdapterInfo {
                    name: name.clone(),
                    status: read_text(&entry.path().join("operstate")),
                    link_speed: read_number(&entry.path().join("speed"))
                        .filter(|speed| *speed > 0)
                        .map(|speed| format!("{speed} Mbps")),
                    mac_address: read_text(&entry.path().join("address")),
                    received_bytes: stats.map(|value| value.total_received()).unwrap_or(0),
                    transmitted_bytes: stats.map(|value| value.total_transmitted()).unwrap_or(0),
                    source: "/sys/class/net and sysinfo".into(),
                });
            }
        }
        if sensors.is_empty() {
            limitations.push(
                "No temperature or fan sensor readings were exposed by the Linux hwmon interfaces."
                    .into(),
            );
        }
        if memory_modules.is_empty() {
            limitations.push("Memory-module SPD/EDAC details were not exposed. Some Linux systems require vendor tools or elevated dmidecode access for DIMM details.".into());
        }
        if drivers.is_empty() {
            limitations.push(
                "PCI driver mapping was unavailable; install pciutils to use lspci -nnk.".into(),
            );
        }
        if batteries.is_empty() {
            limitations.push("No battery was detected through /sys/class/power_supply. Desktop systems commonly have no battery.".into());
        }
        if gpus.is_empty() {
            limitations.push("No graphics device was enumerated through /sys/class/drm. Check kernel drivers or install pciutils for additional inventory.".into());
        }
        if std::path::Path::new("/sys/class/tpm").exists() {
            firmware.tpm_detected = Some(true);
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    limitations.push("This release supports Windows and Linux adapters only.".into());

    if gpus.is_empty() {
        limitations.push("No GPU details were returned by the platform adapter.".into());
    }
    if gpu_telemetry.is_empty() {
        limitations.push("Live GPU telemetry requires a supported vendor provider: nvidia-smi, AMD SMI (amd-smi), or Intel intel_gpu_top on supported Linux hardware. Missing telemetry is not treated as a hardware failure.".into());
    }
    if network_adapters.is_empty() {
        limitations
            .push("No active network interfaces were returned by the platform adapter.".into());
    }
    if problem_devices.is_empty() && cfg!(target_os = "windows") {
        limitations.push("No non-OK Plug and Play devices were returned; this is not a complete physical hardware test.".into());
    }

    let driver_findings = collect_driver_findings(&drivers, &problem_devices);
    Ok(HardwareDiagnostics {
        scanned_at: timestamp_string(),
        cpu_usage_percent: cpu_usage,
        cpu_frequency_mhz: cpu_frequency,
        cpu_cores,
        available_memory_bytes: system.available_memory(),
        memory_modules,
        drivers,
        driver_findings,
        peripherals,
        gpus,
        gpu_telemetry,
        sensors,
        batteries,
        network_adapters,
        problem_devices,
        firmware,
        vendor_support,
        limitations,
    })
}

#[tauri::command]
async fn get_hardware_diagnostics() -> Result<HardwareDiagnostics, String> {
    tauri::async_runtime::spawn_blocking(collect_hardware_diagnostics_blocking)
        .await
        .map_err(|error| format!("Hardware diagnostics worker failed: {error}"))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    test_id: String,
    status: String,
    started_at: String,
    completed_at: String,
    duration_ms: u64,
    metric_value: Option<f64>,
    metric_unit: String,
    summary: String,
    limitations: Vec<String>,
}

fn is_prime_candidate(value: u64) -> bool {
    if value < 2 {
        return false;
    }
    if value == 2 {
        return true;
    }
    if value % 2 == 0 {
        return false;
    }
    let mut divisor = 3_u64;
    while divisor <= value / divisor {
        if value % divisor == 0 {
            return false;
        }
        divisor += 2;
    }
    true
}

fn memory_test_pattern(index: usize) -> u64 {
    (index as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .rotate_left((index % 64) as u32)
        ^ 0xA5A5_5A5A_D3C3_B4B4
}

fn memory_test_pattern_for_pass(index: usize, pass: usize) -> u64 {
    let base = memory_test_pattern(index);
    match pass % 6 {
        0 => base,
        1 => !base,
        2 => base.rotate_left(17) ^ 0xFFFF_0000_AAAA_5555,
        3 => {
            if index % 2 == 0 {
                0xAAAA_AAAA_AAAA_AAAA
            } else {
                0x5555_5555_5555_5555
            }
        }
        4 => 0,
        _ => u64::MAX,
    }
}

fn run_performance_test_blocking(
    test_id: &str,
    duration_seconds: u32,
    memory_mb: u32,
) -> Result<PerformanceTestResult, String> {
    let started_at = timestamp_string();
    let started = Instant::now();
    let (status, metric_value, metric_unit, summary, limitations) = match test_id {
        "cpu_single_core" => {
            if !(3..=15).contains(&duration_seconds) {
                return Err("CPU benchmark duration must be between 3 and 15 seconds.".into());
            }
            let deadline = Instant::now() + Duration::from_secs(duration_seconds as u64);
            let mut candidate = 2_u64;
            let mut primes_found = 0_u64;
            while Instant::now() < deadline {
                if is_prime_candidate(candidate) { primes_found = primes_found.saturating_add(1); }
                candidate = candidate.saturating_add(1);
                if candidate == u64::MAX { break; }
            }
            let elapsed = started.elapsed().as_secs_f64().max(0.001);
            let rate = primes_found as f64 / elapsed;
            (
                "completed".to_string(), Some(rate), "prime candidates / second".to_string(),
                format!("Single-core calculation benchmark completed: {primes_found} prime values found in {:.2} seconds ({rate:.1} per second).", elapsed),
                vec!["This is a short single-thread CPU calculation benchmark, not a multi-core stress test or a standardized comparison score.".into(), "Performance varies with CPU model, power mode, thermal limits, background applications and system load. Monitor temperatures separately.".into()],
            )
        }
        "cpu_multi_core" => {
            if !(3..=10).contains(&duration_seconds) {
                return Err("Multi-core CPU benchmark duration must be between 3 and 10 seconds.".into());
            }
            let workers = std::thread::available_parallelism().map(|count| count.get()).unwrap_or(1).clamp(1, 8);
            let deadline = Instant::now() + Duration::from_secs(duration_seconds as u64);
            let candidates = std::thread::scope(|scope| {
                let mut handles = Vec::with_capacity(workers);
                for worker in 0..workers {
                    handles.push(scope.spawn(move || {
                        let mut candidate = 2_u64 + worker as u64;
                        let mut checked = 0_u64;
                        while Instant::now() < deadline {
                            let _ = is_prime_candidate(candidate);
                            checked = checked.saturating_add(1);
                            candidate = candidate.saturating_add(workers as u64);
                            if candidate == u64::MAX { break; }
                        }
                        checked
                    }));
                }
                handles.into_iter().map(|handle| handle.join().unwrap_or(0)).sum::<u64>()
            });
            let elapsed = started.elapsed().as_secs_f64().max(0.001);
            let rate = candidates as f64 / elapsed;
            (
                "completed".to_string(), Some(rate), "candidate checks / second".to_string(),
                format!("Bounded multi-core CPU benchmark completed using {workers} worker thread(s): {candidates} candidate checks in {:.2} seconds ({rate:.1} checks/second).", elapsed),
                vec!["This is a short CPU calculation benchmark capped at eight worker threads, not a certified stability or thermal stress test.".into(), "Stop other workloads and monitor temperatures separately. Results vary with power mode, CPU scheduling and background activity.".into()],
            )
        }
        "memory_smoke" => {
            if !(32..=512).contains(&memory_mb) {
                return Err("RAM smoke-test allocation must be between 32 and 512 MiB.".into());
            }
            let mut system = System::new();
            system.refresh_memory();
            let requested_bytes = (memory_mb as u64).saturating_mul(1024 * 1024);
            let available_bytes = system.available_memory();
            if available_bytes == 0 || requested_bytes > available_bytes / 4 {
                return Err(format!("The requested {memory_mb} MiB exceeds one quarter of reported available memory ({} MiB). Close applications or select a smaller test size.", available_bytes / (1024 * 1024)));
            }
            let words = usize::try_from(requested_bytes / std::mem::size_of::<u64>() as u64)
                .map_err(|_| "Requested RAM test size is not representable on this platform.".to_string())?;
            let mut buffer = Vec::<u64>::new();
            buffer.try_reserve_exact(words).map_err(|error| format!("Could not reserve {memory_mb} MiB for the RAM smoke test: {error}"))?;
            // Reservation alone does not initialize elements or change Vec::len().
            // Resize only after a successful reservation so indexed write/read passes are safe.
            buffer.resize(words, 0);
            let mut mismatch_index = None;
            for pass in 0..6 {
                for index in 0..words { buffer[index] = memory_test_pattern_for_pass(index, pass); }
                if let Some((index, _)) = buffer.iter().enumerate().find(|(index, value)| **value != memory_test_pattern_for_pass(*index, pass)) {
                    mismatch_index = Some(index);
                    break;
                }
            }
            let verified = mismatch_index.is_none();
            let summary = if verified {
                format!("Allocated {memory_mb} MiB in the application process and completed six write/read-back pattern passes across {words} 64-bit values.")
            } else {
                format!("A read-back mismatch was detected at test word {}. This requires further investigation; it does not identify a specific failed memory module.", mismatch_index.unwrap_or(0))
            };
            (
                if verified { "passed".to_string() } else { "mismatch".to_string() },
                Some(if verified { memory_mb as f64 } else { 0.0 }), "MiB checked".to_string(), summary,
                vec!["This is a bounded user-space memory allocation and six-pattern read-back smoke test. It does not test every physical RAM address, ECC behavior, memory timing or all DIMM slots.".into(), "Use a bootable dedicated memory diagnostic for comprehensive RAM stability testing. The application limits this test to 512 MiB and one quarter of reported available memory.".into()],
            )
        }
        _ => return Err("Unknown performance test. Supported tests are cpu_single_core, cpu_multi_core and memory_smoke.".into()),
    };
    Ok(PerformanceTestResult {
        test_id: test_id.into(),
        status,
        started_at,
        completed_at: timestamp_string(),
        duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
        metric_value,
        metric_unit,
        summary,
        limitations,
    })
}

#[tauri::command]
async fn run_performance_test(
    test_id: String,
    duration_seconds: u32,
    memory_mb: u32,
) -> Result<PerformanceTestResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_performance_test_blocking(&test_id, duration_seconds, memory_mb)
    })
    .await
    .map_err(|error| format!("Performance test worker failed: {error}"))?
}

fn split_table_columns(line: &str) -> Vec<String> {
    let mut columns = Vec::new();
    let mut current = String::new();
    let mut whitespace = 0usize;
    for ch in line.chars() {
        if ch.is_whitespace() {
            whitespace += 1;
        } else {
            if whitespace >= 2 && !current.trim().is_empty() {
                columns.push(current.trim().to_string());
                current.clear();
            } else if whitespace == 1 && !current.is_empty() {
                current.push(' ');
            }
            whitespace = 0;
            current.push(ch);
        }
    }
    if !current.trim().is_empty() {
        columns.push(current.trim().to_string());
    }
    columns
}

fn parse_winget_upgrade_table(output: &str) -> Vec<AvailableUpdate> {
    let lines = output.lines().collect::<Vec<_>>();
    let Some(header_index) = lines
        .iter()
        .position(|line| line.contains("Name") && line.contains("Id") && line.contains("Version"))
    else {
        return Vec::new();
    };
    let mut found_separator = false;
    let mut updates = Vec::new();
    for line in lines.iter().skip(header_index + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !found_separator {
            if trimmed.chars().all(|ch| ch == '-' || ch.is_whitespace()) {
                found_separator = true;
            }
            continue;
        }
        if trimmed.starts_with('\u{2500}') || trimmed.starts_with("No installed package") {
            continue;
        }
        let columns = split_table_columns(trimmed);
        if columns.len() >= 4 {
            updates.push(AvailableUpdate {
                name: columns[0].clone(),
                version: Some(columns[3].clone()),
                source: "WinGet".into(),
                detail: Some(format!(
                    "Package ID: {}; installed version: {}",
                    columns.get(1).cloned().unwrap_or_default(),
                    columns.get(2).cloned().unwrap_or_default()
                )),
                install_id: columns.get(1).cloned(),
            });
        }
    }
    updates
}

fn parse_zypper_update_table(output: &str) -> Vec<AvailableUpdate> {
    let lines = output.lines().collect::<Vec<_>>();
    let Some(header_index) = lines.iter().position(|line| {
        line.contains('|')
            && line.contains("Repository")
            && line.contains("Name")
            && line.contains("Available Version")
    }) else {
        return Vec::new();
    };
    let headers = lines[header_index]
        .split('|')
        .map(|column| column.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let name_index = headers.iter().position(|header| header == "name");
    let version_index = headers
        .iter()
        .position(|header| header.contains("available version"));
    let repository_index = headers.iter().position(|header| header == "repository");
    let (Some(name_index), Some(version_index)) = (name_index, version_index) else {
        return Vec::new();
    };

    lines
        .iter()
        .skip(header_index + 1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed
                    .chars()
                    .all(|ch| ch == '-' || ch == '+' || ch == '|')
            {
                return None;
            }
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            let name = columns.get(name_index).copied().unwrap_or_default();
            let version = columns.get(version_index).copied().unwrap_or_default();
            if name.is_empty() || version.is_empty() || name.eq_ignore_ascii_case("name") {
                return None;
            }
            let repository = repository_index
                .and_then(|index| columns.get(index).copied())
                .filter(|value| !value.is_empty());
            Some(AvailableUpdate {
                name: name.to_string(),
                version: Some(version.to_string()),
                source: "Zypper".into(),
                detail: repository.map(|value| format!("Repository: {value}")),
                install_id: Some(name.to_string()),
            })
        })
        .collect()
}

fn parse_apt_simulation(output: &str) -> Vec<AvailableUpdate> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("Inst ")?;
            let mut parts = rest.split_whitespace();
            let name = parts.next()?;
            let available = parts.find(|part| part.starts_with('(')).map(|part| {
                part.trim_start_matches('(')
                    .trim_end_matches(')')
                    .to_string()
            });
            Some(AvailableUpdate {
                name: name.to_string(),
                version: available.filter(|value| !value.is_empty()),
                source: "APT simulation".into(),
                detail: Some(trimmed.to_string()),
                install_id: Some(name.to_string()),
            })
        })
        .collect()
}

fn parse_dnf_check_update(output: &str) -> Vec<AvailableUpdate> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("last metadata")
                || lower.starts_with("obsoleting packages")
                || lower.starts_with("available upgrades")
                || lower.starts_with("security:")
                || lower.starts_with("error:")
                || lower.starts_with("package ")
            {
                return None;
            }
            let parts = trimmed.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 3
                || parts[0].starts_with('=')
                || !parts[1].chars().any(|ch| ch.is_ascii_digit())
            {
                return None;
            }
            Some(AvailableUpdate {
                name: parts[0].to_string(),
                version: Some(parts[1].to_string()),
                source: "DNF check-update".into(),
                detail: Some(trimmed.to_string()),
                install_id: Some(parts[0].to_string()),
            })
        })
        .collect()
}

fn parse_pacman_upgrade_list(output: &str) -> Vec<AvailableUpdate> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let parts = trimmed.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 4 || parts[2] != "->" {
                return None;
            }
            Some(AvailableUpdate {
                name: parts[0].to_string(),
                version: Some(parts[3].to_string()),
                source: "Pacman -Qu".into(),
                detail: Some(format!("Installed version: {}", parts[1])),
                install_id: Some(parts[0].to_string()),
            })
        })
        .collect()
}

fn collect_windows_update_status() -> Result<UpdateStatus, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
  $session = New-Object -ComObject Microsoft.Update.Session
  $searcher = $session.CreateUpdateSearcher()
  $result = $searcher.Search("IsInstalled=0 and IsHidden=0")
  $items = @()
  for ($i=0; $i -lt [Math]::Min($result.Updates.Count, 100); $i++) {
    $u = $result.Updates.Item($i)
    $kb = @($u.KBArticleIDs) -join ', '
    $items += [pscustomobject]@{ title=$u.Title; version=$null; source='Windows Update Agent'; detail=if($kb){"KB $kb"}else{'Software update'} }
  }
  [pscustomobject]@{ ok=$true; count=$result.Updates.Count; updates=@($items); error=$null } | ConvertTo-Json -Depth 5 -Compress
} catch {
  [pscustomobject]@{ ok=$false; count=$null; updates=@(); error=$_.Exception.Message } | ConvertTo-Json -Depth 5 -Compress
}
"#;
    let output = run_command(
        "powershell.exe",
        &[
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            script.into(),
        ],
        90,
    )?;
    let data: Value = serde_json::from_str(&output.stdout)
        .map_err(|error| format!("Windows Update Agent returned invalid JSON: {error}"))?;
    let mut updates = json_array(data.get("updates"))
        .into_iter()
        .map(|item| AvailableUpdate {
            name: json_string(item.get("title"))
                .unwrap_or_else(|| "Untitled Windows update".into()),
            version: json_string(item.get("version")),
            source: json_string(item.get("source"))
                .unwrap_or_else(|| "Windows Update Agent".into()),
            detail: json_string(item.get("detail")),
            install_id: None,
        })
        .collect::<Vec<_>>();
    let ok = data.get("ok").and_then(Value::as_bool).unwrap_or(false);
    let os_update_count =
        json_number(data.get("count")).map(|value| value.min(u32::MAX as u64) as u32);
    let mut limitations = vec!["This is a read-only Windows Update Agent and WinGet discovery pass. No updates were downloaded or installed.".into(), "Driver and firmware updates may also be distributed through official PC-manufacturer utilities.".into()];
    if let Some(error) = json_string(data.get("error")) {
        limitations.push(format!("Windows Update Agent: {error}"));
    }
    let mut winget_checked = false;
    if executable_exists("winget") {
        match run_command(
            "winget",
            &[
                "list".into(),
                "--upgrade-available".into(),
                "--disable-interactivity".into(),
            ],
            60,
        ) {
            Ok(winget_output) => {
                if winget_output.exit_code == Some(0) {
                    winget_checked = true;
                    let app_updates = parse_winget_upgrade_table(&winget_output.stdout);
                    if app_updates.len() > 100 {
                        limitations.push("WinGet returned more than 100 upgrade candidates; the visible report is capped at 100 combined entries while the count includes all parsed candidates.".into());
                    }
                    updates.extend(app_updates);
                } else {
                    limitations.push(format!("WinGet exited with code {:?}; the application-update list could not be verified.", winget_output.exit_code));
                }
                if !winget_output.stderr.trim().is_empty() {
                    limitations.push(format!("WinGet message: {}", winget_output.stderr.trim()));
                }
            }
            Err(error) => limitations.push(format!(
                "WinGet application update check could not run: {error}"
            )),
        }
    } else {
        limitations.push(
            "WinGet was not found; third-party Windows application updates were not checked."
                .into(),
        );
    }
    let winget_count = updates
        .len()
        .saturating_sub(json_array(data.get("updates")).len()) as u32;
    let total_count = match (os_update_count, winget_checked) {
        (Some(os_count), true) => Some(os_count.saturating_add(winget_count)),
        (Some(os_count), false) => Some(os_count),
        (None, true) => Some(winget_count),
        (None, false) => None,
    };
    if updates.len() > 100 {
        updates.truncate(100);
        limitations.push("The displayed Windows update list is capped at 100 entries; the reported count uses the Windows Update Agent total plus parsed WinGet candidates.".into());
    }
    let overall_ok = ok || winget_checked;
    Ok(UpdateStatus {
        checked_at: timestamp_string(),
        platform: "Windows".into(),
        package_manager: if winget_checked {
            "Windows Update Agent + WinGet".into()
        } else {
            "Windows Update Agent".into()
        },
        status: if !overall_ok {
            "check_error".into()
        } else if !ok {
            "partial".into()
        } else if total_count.unwrap_or(0) > 0 {
            "updates_available".into()
        } else {
            "up_to_date".into()
        },
        update_count: total_count,
        updates,
        limitations,
    })
}

#[cfg(target_os = "linux")]
fn collect_linux_update_status() -> Result<UpdateStatus, String> {
    let choices = [
        (
            "apt-get",
            vec!["-s".to_string(), "upgrade".to_string()],
            "APT",
        ),
        (
            "dnf",
            vec!["-q".to_string(), "check-update".to_string()],
            "DNF",
        ),
        ("pacman", vec!["-Qu".to_string()], "Pacman"),
        (
            "zypper",
            vec!["--non-interactive".to_string(), "list-updates".to_string()],
            "Zypper",
        ),
    ];
    let release = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let preferred = linux_package_manager_from_os_release(&release);
    if preferred.is_none() {
        return Ok(UpdateStatus {
            checked_at: timestamp_string(), platform: "Linux".into(),
            package_manager: "Unsupported distribution family".into(), status: "unsupported_distribution".into(),
            update_count: None, updates: Vec::new(),
            limitations: vec!["CrossDiag could not safely identify this distribution's base package-manager family from /etc/os-release. No package manager was guessed or executed. Use the distribution's official updater.".into()],
        });
    }
    let selected = preferred.and_then(|name| {
        choices
            .iter()
            .find(|(program, _, _)| *program == name && executable_exists(program))
    });
    let Some((program, args, manager)) = selected else {
        return Ok(UpdateStatus { checked_at: timestamp_string(), platform: "Linux".into(), package_manager: "Not detected".into(), status: "unavailable".into(), update_count: None, updates: Vec::new(), limitations: vec!["No supported APT, DNF, Pacman or Zypper package manager was detected for this distribution.".into()] });
    };
    let output = run_command(program, args, 90)?;
    let combined = format!("{}\n{}", output.stdout, output.stderr);
    let mut updates = Vec::new();
    let mut flatpak_checked = false;
    let mut snap_checked = false;
    match *manager {
        "APT" => updates.extend(parse_apt_simulation(&output.stdout)),
        "DNF" => updates.extend(parse_dnf_check_update(&combined)),
        "Pacman" => updates.extend(parse_pacman_upgrade_list(&output.stdout)),
        "Zypper" => {
            updates.extend(parse_zypper_update_table(&output.stdout));
        }
        _ => {}
    }
    let successful =
        output.exit_code == Some(0) || (*manager == "DNF" && output.exit_code == Some(100));
    let mut limitations = vec![format!("Read-only update discovery using {manager}; no packages were installed or changed."), "Package metadata may be stale. Refresh your distribution's package metadata using its normal package manager before relying on the result.".into(), "Firmware updates and vendor-specific updater tools are not included in this check.".into()];
    if executable_exists("flatpak") {
        match run_command(
            "flatpak",
            &[
                "remote-ls".into(),
                "--updates".into(),
                "--columns=application,branch".into(),
            ],
            60,
        ) {
            Ok(flatpak_output) => {
                flatpak_checked = flatpak_output.exit_code == Some(0);
                if flatpak_checked {
                    for line in flatpak_output
                        .stdout
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                    {
                        let parts = line.split_whitespace().collect::<Vec<_>>();
                        if !parts.is_empty() && !parts[0].eq_ignore_ascii_case("application") {
                            updates.push(AvailableUpdate {
                                name: parts[0].into(),
                                version: None,
                                source: "Flatpak".into(),
                                detail: parts.get(1).map(|branch| {
                                    format!(
                                        "Branch: {branch}; update reported by configured remote"
                                    )
                                }),
                                install_id: Some(parts[0].into()),
                            });
                        }
                    }
                }
                if flatpak_output.exit_code != Some(0) {
                    limitations.push(format!(
                        "Flatpak exited with code {:?}; its update list was not verified.",
                        flatpak_output.exit_code
                    ));
                }
                if !flatpak_output.stderr.trim().is_empty() {
                    limitations.push(format!("Flatpak message: {}", flatpak_output.stderr.trim()));
                }
            }
            Err(error) => {
                limitations.push(format!("Flatpak update discovery could not run: {error}"))
            }
        }
    }
    if executable_exists("snap") {
        match run_command("snap", &["refresh".into(), "--list".into()], 45) {
            Ok(snap_output) => {
                snap_checked = snap_output.exit_code == Some(0);
                if snap_checked {
                    let mut after_header = false;
                    for line in snap_output
                        .stdout
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                    {
                        if !after_header {
                            if line.to_ascii_lowercase().contains("name")
                                && line.to_ascii_lowercase().contains("version")
                            {
                                after_header = true;
                            }
                            continue;
                        }
                        let parts = line.split_whitespace().collect::<Vec<_>>();
                        if parts.len() >= 2 && parts[0] != "All" {
                            updates.push(AvailableUpdate {
                                name: parts[0].into(),
                                version: Some(parts[1].into()),
                                source: "Snap".into(),
                                detail: Some(line.trim().into()),
                                install_id: Some(parts[0].into()),
                            });
                        }
                    }
                }
                if snap_output.exit_code != Some(0) {
                    limitations.push(format!(
                        "Snap exited with code {:?}; its update list was not verified.",
                        snap_output.exit_code
                    ));
                }
                if !snap_output.stderr.trim().is_empty() {
                    limitations.push(format!("Snap message: {}", snap_output.stderr.trim()));
                }
            }
            Err(error) => limitations.push(format!("Snap update discovery could not run: {error}")),
        }
    }
    if !output.stderr.trim().is_empty() {
        limitations.push(format!("Package manager message: {}", output.stderr.trim()));
    }
    if !successful {
        limitations.push(format!(
            "The package manager exited with code {:?}.",
            output.exit_code
        ));
    }
    if updates.is_empty() && !successful && combined.trim().is_empty() {
        limitations.push("The package-manager check returned no usable output.".into());
    }
    let count = if successful {
        updates.len().min(u32::MAX as usize) as u32
    } else {
        0
    };
    if updates.len() > 100 {
        updates.truncate(100);
        limitations.push("The displayed update list is capped at 100 entries; the reported count includes all entries collected from successful checks.".into());
    }
    let mut manager_label = (*manager).to_string();
    if flatpak_checked {
        manager_label.push_str(" + Flatpak");
    }
    if snap_checked {
        manager_label.push_str(" + Snap");
    }
    Ok(UpdateStatus {
        checked_at: timestamp_string(),
        platform: "Linux".into(),
        package_manager: manager_label,
        status: if !successful {
            "check_error".into()
        } else if updates.is_empty() {
            "up_to_date".into()
        } else {
            "updates_available".into()
        },
        update_count: if successful { Some(count) } else { None },
        updates,
        limitations,
    })
}

#[cfg(target_os = "windows")]
fn collect_platform_update_status() -> Result<UpdateStatus, String> {
    collect_windows_update_status()
}
#[cfg(target_os = "linux")]
fn collect_platform_update_status() -> Result<UpdateStatus, String> {
    collect_linux_update_status()
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn collect_platform_update_status() -> Result<UpdateStatus, String> {
    Ok(UpdateStatus {
        checked_at: timestamp_string(),
        platform: std::env::consts::OS.into(),
        package_manager: "Unsupported platform".into(),
        status: "unavailable".into(),
        update_count: None,
        updates: Vec::new(),
        limitations: vec!["This build contains Windows and Linux update adapters only.".into()],
    })
}

#[tauri::command]
async fn check_update_status() -> Result<UpdateStatus, String> {
    tauri::async_runtime::spawn_blocking(collect_platform_update_status)
        .await
        .map_err(|error| format!("Update check worker failed: {error}"))?
}

#[cfg(target_os = "windows")]
fn collect_platform_faults() -> Result<FaultReport, String> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [Console]::OutputEncoding
$ErrorActionPreference = 'SilentlyContinue'
$start = (Get-Date).AddDays(-7)
$events = @(Get-WinEvent -FilterHashtable @{LogName='System'; Level=1,2; StartTime=$start} -MaxEvents 50 | ForEach-Object {
  [pscustomobject]@{ timestamp=$_.TimeCreated.ToString('o'); severity=$_.LevelDisplayName; source=$_.ProviderName; code=[string]$_.Id; message=($_.Message -replace '\s+',' ').Substring(0,[Math]::Min(700,($_.Message -replace '\s+',' ').Length)) }
})
@($events) | ConvertTo-Json -Depth 4 -Compress
"#;
    let output = run_command(
        "powershell.exe",
        &[
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            script.into(),
        ],
        30,
    )?;
    let data: Value = if output.stdout.trim().is_empty() {
        Value::Array(Vec::new())
    } else {
        serde_json::from_str(&output.stdout)
            .map_err(|error| format!("Windows event query returned invalid JSON: {error}"))?
    };
    let items = match data {
        Value::Array(values) => values,
        Value::Object(_) => vec![data],
        _ => Vec::new(),
    };
    let faults = items
        .into_iter()
        .map(|item| SystemFault {
            timestamp: json_string(item.get("timestamp")),
            severity: json_string(item.get("severity")).unwrap_or_else(|| "Error".into()),
            source: json_string(item.get("source")).unwrap_or_else(|| "Windows System log".into()),
            code: json_string(item.get("code")),
            message: json_string(item.get("message"))
                .unwrap_or_else(|| "Event message unavailable".into()),
        })
        .collect();
    Ok(FaultReport { scanned_at: timestamp_string(), platform: "Windows".into(), source: "Windows System event log (last seven days)".into(), faults, limitations: vec!["Only the System log's critical and error-level events are included; application logs and minidump analysis are separate tasks.".into(), "An event is evidence, not automatic proof that a specific hardware component is defective.".into()] })
}

#[cfg(target_os = "linux")]
fn collect_platform_faults() -> Result<FaultReport, String> {
    if !executable_exists("journalctl") {
        return Ok(FaultReport { scanned_at: timestamp_string(), platform: "Linux".into(), source: "system journal unavailable".into(), faults: Vec::new(), limitations: vec!["journalctl was not found. Kernel log review may require an administrator-approved dmesg access.".into()] });
    }
    let output = run_command(
        "journalctl",
        &[
            "--no-pager".into(),
            "-p".into(),
            "err".into(),
            "-b".into(),
            "-n".into(),
            "40".into(),
            "-o".into(),
            "json".into(),
        ],
        20,
    )?;
    let mut faults = Vec::new();
    for line in output.stdout.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(item) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let message = json_string(item.get("MESSAGE"))
            .unwrap_or_else(|| "Journal entry has no message".into());
        let timestamp = json_number(item.get("__REALTIME_TIMESTAMP"))
            .map(|micros| format!("unix:{}", micros / 1_000_000));
        faults.push(SystemFault {
            timestamp,
            severity: "Error or higher (journal priority <= 3)".into(),
            source: json_string(item.get("SYSLOG_IDENTIFIER"))
                .or_else(|| json_string(item.get("_SYSTEMD_UNIT")))
                .unwrap_or_else(|| "Linux system journal".into()),
            code: json_string(item.get("MESSAGE_ID")),
            message: message.chars().take(700).collect(),
        });
    }
    let mut limitations = vec!["Only current-boot journal entries with error priority are included; older logs and application-specific logs may need separate review.".into(), "Permission restrictions can hide some kernel and service events.".into(), "Journal errors do not establish a failed component without corroborating tests.".into()];
    if !output.stderr.trim().is_empty() {
        limitations.push(format!("journalctl message: {}", output.stderr.trim()));
    }
    if output.exit_code != Some(0) {
        limitations.push(format!(
            "journalctl exited with code {:?}.",
            output.exit_code
        ));
    }
    Ok(FaultReport {
        scanned_at: timestamp_string(),
        platform: "Linux".into(),
        source: "journalctl current boot, error priority".into(),
        faults,
        limitations,
    })
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn collect_platform_faults() -> Result<FaultReport, String> {
    Ok(FaultReport {
        scanned_at: timestamp_string(),
        platform: std::env::consts::OS.into(),
        source: "Unsupported platform".into(),
        faults: Vec::new(),
        limitations: vec!["This build contains Windows and Linux log adapters only.".into()],
    })
}

#[tauri::command]
async fn analyze_system_faults() -> Result<FaultReport, String> {
    tauri::async_runtime::spawn_blocking(collect_platform_faults)
        .await
        .map_err(|error| format!("Fault analysis worker failed: {error}"))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstallResult {
    package: String,
    source: String,
    started_at: String,
    completed_at: String,
    success: bool,
    exit_code: Option<i32>,
    output: String,
    limitations: Vec<String>,
}

fn valid_package_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ".+:-_@".contains(ch))
        && value.trim() == value
        && !value.contains("..")
}

#[cfg(target_os = "windows")]
fn install_update_blocking(package: &str, source: &str) -> Result<UpdateInstallResult, String> {
    if source != "WinGet" {
        return Err("Only WinGet application updates can be started from CrossDiag. Windows operating-system updates must be completed through Windows Update settings.".into());
    }
    if !valid_package_identifier(package) {
        return Err("The package identifier contains unsupported characters.".into());
    }
    let current = collect_windows_update_status()?;
    if !current
        .updates
        .iter()
        .any(|item| item.source == source && item.install_id.as_deref() == Some(package))
    {
        return Err("This package is no longer present in the latest verified WinGet update list. Run Check for updates again before installing.".into());
    }
    let started_at = timestamp_string();
    let output = run_command(
        "winget",
        &[
            "upgrade".into(),
            "--id".into(),
            package.into(),
            "--exact".into(),
            "--accept-source-agreements".into(),
            "--accept-package-agreements".into(),
            "--disable-interactivity".into(),
        ],
        900,
    )?;
    let success = output.exit_code == Some(0);
    Ok(UpdateInstallResult {
        package: package.into(), source: source.into(), started_at,
        completed_at: timestamp_string(), success, exit_code: output.exit_code,
        output: format!("{}\n{}", output.stdout, output.stderr).chars().take(12000).collect(),
        limitations: vec!["The installation was initiated only after an explicit user action. Installer behavior, restart requirements and vendor-specific prompts depend on the package.".into(), "CrossDiag does not create a system restore point or guarantee installer rollback.".into()],
    })
}

#[cfg(target_os = "linux")]
fn install_update_blocking(package: &str, source: &str) -> Result<UpdateInstallResult, String> {
    if !valid_package_identifier(package) {
        return Err("The package identifier contains unsupported characters.".into());
    }
    let current = collect_linux_update_status()?;
    if !current
        .updates
        .iter()
        .any(|item| item.source == source && item.install_id.as_deref() == Some(package))
    {
        return Err("This package is no longer present in the latest verified package-manager update list. Run Check for updates again before installing.".into());
    }
    let (program, mut args): (&str, Vec<String>) = match source {
        "APT simulation" | "APT" => ("apt-get", vec!["install".into(), "--only-upgrade".into(), "-y".into(), package.into()]),
        "DNF check-update" | "DNF" => ("dnf", vec!["upgrade".into(), "-y".into(), package.into()]),
        "Pacman -Qu" | "Pacman" => ("pacman", vec!["-S".into(), "--needed".into(), "--noconfirm".into(), package.into()]),
        "Zypper" => ("zypper", vec!["--non-interactive".into(), "update".into(), package.into()]),
        "Flatpak" => ("flatpak", vec!["update".into(), "-y".into(), "--noninteractive".into(), package.into()]),
        "Snap" => ("snap", vec!["refresh".into(), package.into()]),
        _ => return Err("This update source is not supported for installation. Refresh the update list and select a package from a supported package manager.".into()),
    };
    let started_at = timestamp_string();
    // Flatpak's per-user installation must stay in the logged-in user's context;
    // system-wide operations can trigger their own PolicyKit authorization.
    let is_flatpak = source == "Flatpak";
    let is_root = run_command("id", &["-u".into()], 5)
        .ok()
        .map(|result| result.stdout.trim() == "0")
        .unwrap_or(false);
    let (executable, final_args) = if is_root || is_flatpak {
        (program.to_string(), args)
    } else if executable_exists("pkexec") {
        let mut elevated = vec![program.to_string()];
        elevated.append(&mut args);
        ("pkexec".to_string(), elevated)
    } else {
        return Err("Administrator authorization is required, but pkexec is not available. Use your distribution's package manager or install a PolicyKit authentication agent.".into());
    };
    let output = run_command(&executable, &final_args, 900)?;
    let success = output.exit_code == Some(0);
    Ok(UpdateInstallResult {
        package: package.into(), source: source.into(), started_at,
        completed_at: timestamp_string(), success, exit_code: output.exit_code,
        output: format!("{}\n{}", output.stdout, output.stderr).chars().take(12000).collect(),
        limitations: vec!["The package manager may update dependencies required by the selected package.".into(), "CrossDiag does not guarantee rollback. Back up important data and review distribution notices before installing updates.".into()],
    })
}

#[cfg(target_os = "linux")]
fn install_linux_distribution_updates() -> Result<UpdateInstallResult, String> {
    let discovered = collect_linux_update_status()?;
    let has_distribution_updates = discovered.updates.iter().any(|item| {
        matches!(
            item.source.as_str(),
            "APT simulation" | "DNF check-update" | "Pacman -Qu" | "Zypper"
        )
    });
    if !has_distribution_updates {
        return Err("No base-distribution package updates were found in the latest check. Flatpak/Snap updates are separate; refresh package metadata and check again before upgrading the OS packages.".into());
    }
    let release = fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let id = release
        .lines()
        .find_map(|line| line.strip_prefix("id="))
        .unwrap_or_default()
        .trim_matches('"');
    let id_like = release
        .lines()
        .find_map(|line| line.strip_prefix("id_like="))
        .unwrap_or_default()
        .trim_matches('"');
    let family = format!("{id} {id_like}");
    let choices: Vec<(&str, Vec<String>)> = if [
        "ubuntu",
        "debian",
        "linuxmint",
        "pop",
        "elementary",
        "kali",
        "raspbian",
    ]
    .iter()
    .any(|item| family.contains(item))
    {
        vec![("apt-get", vec!["upgrade".into(), "-y".into()])]
    } else if ["fedora", "rhel", "centos", "rocky", "almalinux", "ol"]
        .iter()
        .any(|item| family.contains(item))
    {
        vec![("dnf", vec!["upgrade".into(), "-y".into()])]
    } else if ["arch", "manjaro", "endeavouros"]
        .iter()
        .any(|item| family.contains(item))
    {
        vec![("pacman", vec!["-Syu".into(), "--noconfirm".into()])]
    } else if ["opensuse", "suse", "sles"]
        .iter()
        .any(|item| family.contains(item))
    {
        vec![(
            "zypper",
            vec!["--non-interactive".into(), "update".into(), "-y".into()],
        )]
    } else {
        return Err("The Linux distribution could not be mapped safely to a supported package-manager upgrade command. Use its official graphical updater or package-manager documentation.".into());
    };
    let Some((program, args)) = choices.into_iter().next() else {
        return Err("No safe distribution upgrade command was selected. Use the distribution's official graphical updater.".into());
    };
    if !executable_exists(program) {
        return Err(format!(
            "The expected package manager {program} is not installed."
        ));
    }
    let started_at = timestamp_string();
    let is_root = run_command("id", &["-u".into()], 5)
        .ok()
        .map(|result| result.stdout.trim() == "0")
        .unwrap_or(false);
    let (executable, final_args) = if is_root {
        (program.to_string(), args)
    } else if executable_exists("pkexec") {
        let mut elevated = vec![program.to_string()];
        elevated.extend(args);
        ("pkexec".to_string(), elevated)
    } else {
        return Err("Administrator authorization is required, but pkexec is not available. Use your distribution's graphical updater.".into());
    };
    let output = run_command(&executable, &final_args, 1800)?;
    Ok(UpdateInstallResult {
        package: "All distribution packages".into(), source: program.into(), started_at, completed_at: timestamp_string(),
        success: output.exit_code == Some(0), exit_code: output.exit_code,
        output: format!("{}\n{}", output.stdout, output.stderr).chars().take(16000).collect(),
        limitations: vec!["This was a user-confirmed distribution-wide upgrade using the normal package manager. Package removals are not requested by the selected APT upgrade command; package-manager behavior varies by distribution.".into(), "A reboot may be required, especially after kernel, firmware or core-library updates. CrossDiag does not automatically reboot.".into(), "Flatpak and Snap application updates are separate from this distribution package transaction.".into()],
    })
}

#[cfg(not(target_os = "linux"))]
fn install_linux_distribution_updates() -> Result<UpdateInstallResult, String> {
    Err("Distribution-wide package upgrades are available only on Linux builds. Use the operating system's official update settings.".into())
}

#[tauri::command]
async fn install_operating_system_updates() -> Result<UpdateInstallResult, String> {
    tauri::async_runtime::spawn_blocking(install_linux_distribution_updates)
        .await
        .map_err(|error| format!("Operating-system update worker failed: {error}"))?
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn install_update_blocking(_package: &str, _source: &str) -> Result<UpdateInstallResult, String> {
    Err("Update installation is supported only on Windows and Linux builds.".into())
}

#[tauri::command]
async fn install_selected_update(
    package: String,
    source: String,
) -> Result<UpdateInstallResult, String> {
    tauri::async_runtime::spawn_blocking(move || install_update_blocking(&package, &source))
        .await
        .map_err(|error| format!("Update installation worker failed: {error}"))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAction {
    id: String,
    title: String,
    description: String,
    category: String,
    is_read_only: bool,
    requires_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    action_id: String,
    title: String,
    started_at: String,
    completed_at: String,
    success: bool,
    exit_code: Option<i32>,
    output: String,
    limitations: Vec<String>,
}

#[tauri::command]
fn get_repair_catalog() -> Vec<RepairAction> {
    #[cfg(target_os = "windows")]
    {
        return vec![
        RepairAction { id: "flush_dns".into(), title: "Flush DNS cache".into(), description: "Clear the local DNS resolver cache. This does not change DNS server settings.".into(), category: "Network".into(), is_read_only: false, requires_admin: false },
        RepairAction { id: "verify_system_files".into(), title: "Verify Windows system files".into(), description: "Run System File Checker in verify-only mode; it reports integrity problems but does not repair files.".into(), category: "Windows health".into(), is_read_only: true, requires_admin: true },
        RepairAction { id: "check_component_store".into(), title: "Check component store".into(), description: "Run DISM CheckHealth. This checks the component store and does not perform RestoreHealth.".into(), category: "Windows health".into(), is_read_only: true, requires_admin: true },
        RepairAction { id: "list_failed_services".into(), title: "Review stopped automatic services".into(), description: "List automatic services that are not running for technician review.".into(), category: "Services".into(), is_read_only: true, requires_admin: false },
    ];
    }
    #[cfg(target_os = "linux")]
    {
        let mut actions = vec![
            RepairAction {
                id: "flush_dns".into(),
                title: "Flush DNS cache".into(),
                description:
                    "Ask systemd-resolved to clear its local cache if that service is available."
                        .into(),
                category: "Network".into(),
                is_read_only: false,
                requires_admin: false,
            },
            RepairAction {
                id: "list_failed_services".into(),
                title: "List failed system services".into(),
                description:
                    "Read the systemd failed-unit list. This action does not restart services."
                        .into(),
                category: "Services".into(),
                is_read_only: true,
                requires_admin: false,
            },
        ];
        if executable_exists("dpkg") || executable_exists("rpm") {
            actions.push(RepairAction { id: "check_package_state".into(), title: "Check package database".into(), description: "Run a non-installing package consistency check using the installed package database.".into(), category: "Packages".into(), is_read_only: true, requires_admin: false });
        }
        return actions;
    }
    #[allow(unreachable_code)]
    Vec::new()
}

fn run_repair_action_blocking(action_id: &str) -> Result<RepairResult, String> {
    #[cfg(target_os = "windows")]
    if ["verify_system_files", "check_component_store"].contains(&action_id) {
        let script = "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)";
        let elevated = run_command(
            "powershell.exe",
            &[
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                script.into(),
            ],
            8,
        )
        .map(|result| result.stdout.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);
        if !elevated {
            return Err("This read-only Windows integrity check requires CrossDiag to be started as Administrator. Close the app, reopen it using Run as administrator, and try again.".into());
        }
    }
    if std::env::consts::OS == "linux"
        && action_id == "flush_dns"
        && !executable_exists("resolvectl")
    {
        return Err("resolvectl is not installed. This system may use another DNS caching service; CrossDiag will not guess which service to restart.".into());
    }
    let started_at = timestamp_string();
    let (title, program, args, limitation) = match (std::env::consts::OS, action_id) {
        ("windows", "flush_dns") => ("Flush DNS cache", "ipconfig", vec!["/flushdns".into()], "Clears only the local DNS cache."),
        ("windows", "verify_system_files") => ("Verify Windows system files", "sfc", vec!["/verifyonly".into()], "Verification only; no system-file repair is performed."),
        ("windows", "check_component_store") => ("Check component store", "dism", vec!["/Online".into(), "/Cleanup-Image".into(), "/CheckHealth".into()], "CheckHealth does not repair the component store."),
        ("windows", "list_failed_services") => ("Review stopped automatic services", "powershell.exe", vec!["-NoProfile".into(), "-NonInteractive".into(), "-Command".into(), "Get-CimInstance Win32_Service | Where-Object { $_.StartMode -eq 'Auto' -and $_.State -ne 'Running' } | Select-Object -First 100 Name,DisplayName,State | Format-Table -AutoSize | Out-String".into()], "Service states are evidence; some services are intentionally stopped."),
        ("linux", "flush_dns") => ("Flush DNS cache", "resolvectl", vec!["flush-caches".into()], "Requires systemd-resolved; other DNS caching services need their own supported workflow."),
        ("linux", "list_failed_services") => ("List failed system services", "systemctl", vec!["--failed".into(), "--no-pager".into()], "Reads systemd failed units and does not restart them."),
        ("linux", "check_package_state") if executable_exists("dpkg") => ("Check package database", "dpkg", vec!["--audit".into()], "dpkg --audit is a read-only package database inspection."),
        ("linux", "check_package_state") if executable_exists("rpm") => ("Check package database", "rpm", vec!["-Va".into()], "rpm verification may take time and reports file differences, not necessarily damage."),
        (_, _) => return Err("This action is not available on the current operating system.".into()),
    };
    let output = run_command(program, &args, 300)?;
    Ok(RepairResult {
        action_id: action_id.into(), title: title.into(), started_at, completed_at: timestamp_string(),
        success: output.exit_code == Some(0), exit_code: output.exit_code,
        output: format!("{}\n{}", output.stdout, output.stderr).chars().take(16000).collect(),
        limitations: vec![limitation.into(), "Review the output before deciding on any separate repair. CrossDiag did not apply a broad automatic repair.".into()],
    })
}

#[tauri::command]
async fn run_repair_action(action_id: String) -> Result<RepairResult, String> {
    let allowed = get_repair_catalog()
        .iter()
        .any(|action| action.id == action_id);
    if !allowed {
        return Err("Unknown or unsupported repair action.".into());
    }
    tauri::async_runtime::spawn_blocking(move || run_repair_action_blocking(&action_id))
        .await
        .map_err(|error| format!("Repair action worker failed: {error}"))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicianCase {
    id: i64,
    title: String,
    device: String,
    status: String,
    notes: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSchedule {
    enabled: bool,
    interval_hours: u32,
    #[serde(default = "default_storage_threshold")]
    threshold_percent: u8,
}

fn default_storage_threshold() -> u8 {
    90
}

fn open_case_database(app: &AppHandle) -> Result<Connection, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app-data directory: {error}"))?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create app-data directory: {error}"))?;
    let connection = Connection::open(directory.join("crossdiag.sqlite3"))
        .map_err(|error| format!("Could not open local SQLite database: {error}"))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| format!("Could not configure SQLite busy timeout: {error}"))?;
    connection
        .execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;
      CREATE TABLE IF NOT EXISTS storage_scans (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        scanned_at TEXT NOT NULL,
        scan_json TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS technician_cases (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        device TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL DEFAULT 'open',
        notes TEXT NOT NULL DEFAULT '',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    ",
        )
        .map_err(|error| format!("Could not initialize SQLite schema: {error}"))?;
    Ok(connection)
}

#[tauri::command]
fn list_technician_cases(app: AppHandle) -> Result<Vec<TechnicianCase>, String> {
    let connection = open_case_database(&app)?;
    let mut statement = connection.prepare("SELECT id,title,device,status,notes,created_at,updated_at FROM technician_cases ORDER BY updated_at DESC LIMIT 500").map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(TechnicianCase {
                id: row.get(0)?,
                title: row.get(1)?,
                device: row.get(2)?,
                status: row.get(3)?,
                notes: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read technician cases: {error}"))
}

#[tauri::command]
fn create_technician_case(
    app: AppHandle,
    title: String,
    device: String,
    notes: String,
) -> Result<TechnicianCase, String> {
    let title = title.trim().to_string();
    let device = device.trim().to_string();
    let notes = notes.trim().to_string();
    if title.is_empty() || title.len() > 120 {
        return Err("Case title must contain 1–120 characters.".into());
    }
    if device.len() > 160 || notes.len() > 8000 {
        return Err("Device field or notes exceed the allowed length.".into());
    }
    let now = timestamp_string();
    let connection = open_case_database(&app)?;
    connection.execute("INSERT INTO technician_cases (title,device,status,notes,created_at,updated_at) VALUES (?1,?2,'open',?3,?4,?4)", params![title,device,notes,now]).map_err(|error| format!("Could not create case: {error}"))?;
    let id = connection.last_insert_rowid();
    Ok(TechnicianCase {
        id,
        title,
        device,
        status: "open".into(),
        notes,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
fn update_technician_case(
    app: AppHandle,
    id: i64,
    status: String,
    notes: String,
) -> Result<(), String> {
    if id <= 0 || !["open", "in_progress", "resolved", "archived"].contains(&status.as_str()) {
        return Err("Invalid case identifier or status.".into());
    }
    if notes.len() > 8000 {
        return Err("Case notes exceed 8,000 characters.".into());
    }
    let connection = open_case_database(&app)?;
    let changed = connection
        .execute(
            "UPDATE technician_cases SET status=?1, notes=?2, updated_at=?3 WHERE id=?4",
            params![status, notes, timestamp_string(), id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("Case not found.".into());
    }
    Ok(())
}

#[tauri::command]
fn delete_technician_case(app: AppHandle, id: i64) -> Result<(), String> {
    if id <= 0 {
        return Err("Invalid case identifier.".into());
    }
    let connection = open_case_database(&app)?;
    let changed = connection
        .execute("DELETE FROM technician_cases WHERE id=?1", params![id])
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("Case not found.".into());
    }
    Ok(())
}

#[tauri::command]
fn get_scan_schedule(app: AppHandle) -> Result<ScanSchedule, String> {
    let connection = open_case_database(&app)?;
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_settings WHERE key='scan_schedule'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let schedule = value
        .and_then(|json| serde_json::from_str::<ScanSchedule>(&json).ok())
        .unwrap_or(ScanSchedule {
            enabled: false,
            interval_hours: 24,
            threshold_percent: 90,
        });
    Ok(ScanSchedule {
        enabled: schedule.enabled,
        interval_hours: schedule.interval_hours.clamp(1, 168),
        threshold_percent: schedule.threshold_percent.min(100),
    })
}

#[tauri::command]
fn save_scan_schedule(
    app: AppHandle,
    enabled: bool,
    interval_hours: u32,
    threshold_percent: u8,
) -> Result<ScanSchedule, String> {
    if !(1..=168).contains(&interval_hours) {
        return Err("Scheduled scan interval must be between 1 and 168 hours.".into());
    }
    if threshold_percent > 100 {
        return Err("Storage threshold must be between 0 and 100 percent.".into());
    }
    let schedule = ScanSchedule {
        enabled,
        interval_hours,
        threshold_percent,
    };
    let json = serde_json::to_string(&schedule).map_err(|error| error.to_string())?;
    let connection = open_case_database(&app)?;
    connection.execute("INSERT INTO app_settings (key,value) VALUES ('scan_schedule',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![json]).map_err(|error| error.to_string())?;
    Ok(schedule)
}

fn unix_timestamp_from_label(value: &str) -> Option<u64> {
    value.strip_prefix("unix:")?.parse::<u64>().ok()
}

fn scheduled_scan_is_due(
    enabled: bool,
    interval_hours: u32,
    last_scan_epoch: Option<u64>,
    now_epoch: u64,
) -> bool {
    if !enabled {
        return false;
    }
    let Some(last_scan) = last_scan_epoch else {
        return true;
    };
    now_epoch
        >= last_scan.saturating_add((interval_hours.clamp(1, 168) as u64).saturating_mul(3600))
}

fn start_background_storage_scheduler(app: AppHandle) {
    let spawn_result = thread::Builder::new().name("crossdiag-storage-scheduler".into()).spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(30));
            let schedule = match get_scan_schedule(app.clone()) {
                Ok(value) if value.enabled => value,
                _ => continue,
            };
            let history = match get_storage_history(app.clone()) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("CrossDiag scheduled scan skipped because local history could not be read: {error}");
                    continue;
                }
            };
            let last_scan = history.last().and_then(|scan| unix_timestamp_from_label(&scan.scanned_at));
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or_default();
            if scheduled_scan_is_due(schedule.enabled, schedule.interval_hours, last_scan, now) {
                if let Err(error) = scan_storage_health_blocking(app.clone(), Some(schedule.threshold_percent)) {
                    eprintln!("CrossDiag scheduled storage scan failed: {error}");
                }
            }
        }
    });
    if let Err(error) = spawn_result {
        eprintln!("CrossDiag background storage scheduler could not start: {error}");
    }
}

#[tauri::command]
fn open_driver_manager() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("mmc.exe")
            .arg("devmgmt.msc")
            .spawn()
            .map_err(|error| format!("Could not open Device Manager: {error}"))?;
        return Ok("Windows Device Manager opened. Review device status and obtain drivers only from Windows Update or the PC/device manufacturer's official source.".into());
    }
    #[cfg(target_os = "linux")]
    {
        return Ok("Linux driver management is distribution- and desktop-specific. Review the detected driver inventory, then use your distribution's official driver/software utility or the hardware manufacturer's support page. CrossDiag does not replace kernel modules or install unsigned drivers.".into());
    }
    #[allow(unreachable_code)]
    Err("Driver manager is unsupported on this platform.".into())
}

#[tauri::command]
fn open_system_update_settings() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd.exe")
            .args(["/C", "start", "", "ms-settings:windowsupdate"])
            .spawn()
            .map_err(|error| format!("Could not open Windows Update settings: {error}"))?;
        return Ok("Windows Update settings opened. Review pending updates and restart requirements there.".into());
    }
    #[cfg(target_os = "linux")]
    {
        return Ok("Install operating-system updates through your distribution's normal package manager or graphical updater. CrossDiag can install a selected package update, but a distribution-wide upgrade should be reviewed in the system's own update tool.".into());
    }
    #[allow(unreachable_code)]
    Err("Operating-system update settings are unavailable on this platform.".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            start_background_storage_scheduler(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_snapshot,
            get_runtime_readiness,
            scan_storage_health,
            get_storage_history,
            get_hardware_diagnostics,
            run_performance_test,
            check_update_status,
            analyze_system_faults,
            install_selected_update,
            install_operating_system_updates,
            get_repair_catalog,
            run_repair_action,
            list_technician_cases,
            create_technician_case,
            update_technician_case,
            delete_technician_case,
            get_scan_schedule,
            save_scan_schedule,
            open_driver_manager,
            open_system_update_settings,
            inspect_recovery_image,
            list_recovery_media
        ])
        .run(tauri::generate_context!())
        .expect("error while running Arcange CrossDiag");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_package_manager_detection_does_not_fall_back_to_an_unrelated_manager() {
        assert_eq!(
            linux_package_manager_from_os_release("ID=ubuntu\nID_LIKE=debian\n"),
            Some("apt-get")
        );
        assert_eq!(
            linux_package_manager_from_os_release("ID=fedora\n"),
            Some("dnf")
        );
        assert_eq!(
            linux_package_manager_from_os_release("ID=arch\n"),
            Some("pacman")
        );
        assert_eq!(
            linux_package_manager_from_os_release("ID=opensuse-tumbleweed\nID_LIKE=suse\n"),
            Some("zypper")
        );
        assert_eq!(
            linux_package_manager_from_os_release("ID=unknown_custom\n"),
            None
        );
    }

    #[test]
    fn runtime_readiness_reports_a_supported_platform_catalog() {
        let readiness = collect_runtime_readiness();
        assert!(!readiness.app_version.is_empty());
        assert!(!readiness.platform.is_empty());
        assert!(!readiness.architecture.is_empty());
        if cfg!(target_os = "windows") {
            assert!(readiness.tools.iter().any(|tool| tool.id == "winget"));
        } else if cfg!(target_os = "linux") {
            assert!(readiness.tools.iter().any(|tool| tool.id == "smartctl"));
            assert!(readiness.tools.iter().any(|tool| tool.id == "lsblk"));
        }
        assert!(readiness.tools.iter().all(|tool| !tool.label.is_empty()
            && !tool.purpose.is_empty()
            && !tool.install_hint.is_empty()));
    }

    #[test]
    fn legacy_storage_history_migrates_latest_records_once_and_preserves_source_file() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!(
            "crossdiag-legacy-history-{}-{}.json",
            std::process::id(),
            timestamp_string().replace(':', "-")
        ));
        let mut legacy = fs::File::create(&path).expect("create legacy history fixture");
        let entries = (0..105).map(sample_storage_scan).collect::<Vec<_>>();
        legacy
            .write_all(
                serde_json::to_string(&entries)
                    .expect("serialize legacy history")
                    .as_bytes(),
            )
            .expect("write legacy history");
        drop(legacy);

        let mut connection = Connection::open_in_memory().expect("open in-memory SQLite database");
        connection.execute_batch("CREATE TABLE storage_scans (id INTEGER PRIMARY KEY AUTOINCREMENT, scanned_at TEXT NOT NULL, scan_json TEXT NOT NULL); CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);").expect("create migration tables");
        migrate_legacy_storage_history_file(&mut connection, &path)
            .expect("migrate legacy history");
        let migrated = read_storage_history(&connection).expect("read migrated history");
        assert_eq!(migrated.len(), 100);
        assert_eq!(
            migrated.first().map(|scan| scan.scanned_at.as_str()),
            Some("unix:5")
        );
        assert_eq!(
            migrated.last().map(|scan| scan.scanned_at.as_str()),
            Some("unix:104")
        );
        assert!(
            path.exists(),
            "legacy JSON should be retained as a migration backup"
        );

        migrate_legacy_storage_history_file(&mut connection, &path)
            .expect("repeat migration check");
        assert_eq!(
            read_storage_history(&connection)
                .expect("read after repeat")
                .len(),
            100
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn malformed_legacy_storage_history_is_not_marked_as_migrated() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!(
            "crossdiag-legacy-bad-{}-{}.json",
            std::process::id(),
            timestamp_string().replace(':', "-")
        ));
        let mut legacy = fs::File::create(&path).expect("create malformed history fixture");
        legacy
            .write_all(b"not-json")
            .expect("write malformed history");
        drop(legacy);
        let mut connection = Connection::open_in_memory().expect("open in-memory SQLite database");
        connection.execute_batch("CREATE TABLE storage_scans (id INTEGER PRIMARY KEY AUTOINCREMENT, scanned_at TEXT NOT NULL, scan_json TEXT NOT NULL); CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);").expect("create migration tables");
        assert!(migrate_legacy_storage_history_file(&mut connection, &path).is_err());
        let marker: Option<String> = connection
            .query_row(
                "SELECT value FROM app_settings WHERE key='storage_history_json_migrated'",
                [],
                |row| row.get(0),
            )
            .optional()
            .expect("query migration marker");
        assert_eq!(marker, None);
        assert!(
            path.exists(),
            "malformed legacy JSON should remain available for recovery"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn oversized_legacy_storage_history_is_not_read_or_marked_as_migrated() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!(
            "crossdiag-legacy-large-{}-{}.json",
            std::process::id(),
            timestamp_string().replace(':', "-")
        ));
        let mut legacy = fs::File::create(&path).expect("create oversized history fixture");
        legacy
            .write_all(&vec![b' '; 16 * 1024 * 1024 + 1])
            .expect("write oversized history");
        drop(legacy);
        let mut connection = Connection::open_in_memory().expect("open in-memory SQLite database");
        connection.execute_batch("CREATE TABLE storage_scans (id INTEGER PRIMARY KEY AUTOINCREMENT, scanned_at TEXT NOT NULL, scan_json TEXT NOT NULL); CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);").expect("create migration tables");
        let error = migrate_legacy_storage_history_file(&mut connection, &path)
            .expect_err("oversized migration should be rejected");
        assert!(error.contains("safety limit"));
        let marker: Option<String> = connection
            .query_row(
                "SELECT value FROM app_settings WHERE key='storage_history_json_migrated'",
                [],
                |row| row.get(0),
            )
            .optional()
            .expect("query migration marker");
        assert_eq!(marker, None);
        assert!(
            path.exists(),
            "oversized legacy JSON should remain available for manual recovery"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn command_output_reader_handles_non_utf8_device_text() {
        let bytes = vec![b'd', b'i', b's', b'k', b'-', 0xff, b'1'];
        let decoded =
            read_pipe(Some(std::io::Cursor::new(bytes))).expect("decode lossy command output");
        assert_eq!(decoded, "disk-\u{fffd}1");
    }

    #[test]
    fn command_output_reader_returns_empty_when_pipe_is_missing() {
        let decoded =
            read_pipe::<std::io::Cursor<Vec<u8>>>(None).expect("empty pipe should be valid");
        assert!(decoded.is_empty());
    }

    #[test]
    fn command_output_reader_enforces_output_size_limit_and_drains_input() {
        let bytes = vec![b'x'; 8 * 1024 * 1024 + 4096];
        let total = bytes.len() as u64;
        let mut cursor = std::io::Cursor::new(bytes);
        let error = read_pipe(Some(&mut cursor)).expect_err("oversized output must be rejected");
        assert!(error.contains("safety limit"));
        assert_eq!(
            cursor.position(),
            total,
            "reader must drain beyond the retained-output cap"
        );
    }

    fn sample_storage_scan(index: u64) -> StorageScan {
        StorageScan {
            scanned_at: format!("unix:{index}"),
            threshold_percent: 90,
            smartctl_available: true,
            smartctl_version: Some("smartctl 7.5".into()),
            scan_status: "completed".into(),
            disks: Vec::new(),
            limitations: Vec::new(),
        }
    }

    #[test]
    fn sqlite_storage_history_keeps_latest_100_scans_in_chronological_order() {
        let mut connection = Connection::open_in_memory().expect("open in-memory SQLite database");
        connection.execute_batch("CREATE TABLE storage_scans (id INTEGER PRIMARY KEY AUTOINCREMENT, scanned_at TEXT NOT NULL, scan_json TEXT NOT NULL);").expect("create storage history table");
        for index in 0..105 {
            insert_storage_history(&mut connection, &sample_storage_scan(index))
                .expect("insert storage scan");
        }
        let history = read_storage_history(&connection).expect("read storage history");
        assert_eq!(history.len(), 100);
        assert_eq!(
            history.first().map(|scan| scan.scanned_at.as_str()),
            Some("unix:5")
        );
        assert_eq!(
            history.last().map(|scan| scan.scanned_at.as_str()),
            Some("unix:104")
        );
    }

    #[test]
    fn unknown_string_has_a_clear_label() {
        assert_eq!(string_or_unknown(None), "Not reported by OS");
        assert_eq!(string_or_unknown(Some("  ".into())), "Not reported by OS");
        assert_eq!(string_or_unknown(Some("Linux".into())), "Linux");
    }

    #[test]
    fn architecture_is_not_empty() {
        assert!(!std::env::consts::ARCH.is_empty());
    }

    #[test]
    fn prime_test_helper_identifies_small_primes() {
        assert!(!is_prime_candidate(0));
        assert!(!is_prime_candidate(1));
        assert!(is_prime_candidate(2));
        assert!(is_prime_candidate(3));
        assert!(!is_prime_candidate(9));
        assert!(is_prime_candidate(97));
    }

    #[test]
    fn memory_test_pattern_is_deterministic_and_index_sensitive() {
        assert_eq!(memory_test_pattern(12), memory_test_pattern(12));
        assert_ne!(memory_test_pattern(12), memory_test_pattern(13));
    }

    #[test]
    fn package_identifiers_allow_expected_package_manager_characters() {
        for value in [
            "Example.App",
            "org.example.App",
            "package-name",
            "package_name",
            "package:amd64",
            "vendor+tool",
        ] {
            assert!(
                valid_package_identifier(value),
                "expected package identifier to be accepted: {value}"
            );
        }
    }

    #[test]
    fn package_identifiers_reject_shell_and_option_injection_characters() {
        for value in [
            "",
            "-y",
            "package name",
            "package;rm",
            "package&&cmd",
            "package|cmd",
            "../package",
            "package..name",
            " package",
            "package ",
            "$(id)",
        ] {
            assert!(
                !valid_package_identifier(value),
                "expected package identifier to be rejected: {value}"
            );
        }
    }

    #[test]
    fn nvme_endurance_is_percentage_used_not_percentage_remaining() {
        let used = 7.0_f64;
        let remaining = (100.0 - used).clamp(0.0, 100.0);
        assert_eq!(remaining, 93.0);
        assert!(remaining > 90.0);
    }

    #[test]
    fn endurance_threshold_is_inclusive_at_ninety() {
        let (status_91, _) = evaluate_disk_status(
            Some(true),
            Some(0),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(91.0),
            90,
        );
        let (status_90, _) = evaluate_disk_status(
            Some(true),
            Some(0),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(90.0),
            90,
        );
        assert_eq!(status_91, "healthy");
        assert_eq!(status_90, "attention");
    }

    #[test]
    fn endurance_threshold_keeps_fractional_percentages() {
        let (status_90_1, _) = evaluate_disk_status(
            Some(true),
            Some(0),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(90.1),
            90,
        );
        let (status_90_0, _) = evaluate_disk_status(
            Some(true),
            Some(0),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(90.0),
            90,
        );
        assert_eq!(status_90_1, "healthy");
        assert_eq!(status_90_0, "attention");
    }

    #[test]
    fn failed_smart_status_overrides_good_endurance_percentage() {
        let (status, _) = evaluate_disk_status(
            Some(false),
            Some(0),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(99.0),
            90,
        );
        assert_eq!(status, "failed");
    }

    #[test]
    fn unknown_endurance_is_not_fabricated() {
        let (status, _) = evaluate_disk_status(
            Some(true),
            Some(0),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            None,
            90,
        );
        assert_eq!(status, "healthy");
    }

    #[test]
    fn findings_explain_threshold_and_media_error_evidence() {
        let findings = collect_disk_findings(
            Some(true),
            Some(1 << 2),
            Some(3),
            Some(8),
            Some(10),
            Some(2),
            Some(1),
            Some(0),
            Some(88.5),
            90,
        );
        assert!(findings
            .iter()
            .any(|finding| finding.contains("degraded reliability")));
        assert!(findings
            .iter()
            .any(|finding| finding.contains("3 NVMe media/data-integrity error")));
        assert!(findings.iter().any(|finding| finding.contains("88.5%")));
        assert!(findings
            .iter()
            .any(|finding| finding.contains("reallocated sector")));
        assert!(findings
            .iter()
            .any(|finding| finding.contains("pending sector")));
    }

    #[test]
    fn nvme_read_only_warning_is_a_failed_state() {
        let (status, _) = evaluate_disk_status(
            Some(true),
            Some(1 << 3),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(99.0),
            90,
        );
        assert_eq!(status, "failed");
    }

    #[test]
    fn nvme_reliability_degraded_warning_needs_attention_not_failed() {
        let (status, _) = evaluate_disk_status(
            Some(true),
            Some(1 << 2),
            Some(0),
            Some(100),
            Some(10),
            Some(0),
            Some(0),
            Some(0),
            Some(99.0),
            90,
        );
        assert_eq!(status, "attention");
    }

    #[test]
    fn negative_endurance_is_ignored_as_invalid() {
        let json = r#"{"smart_status":{"passed":true},"endurance_used":{"current_percent":-5}}"#;
        let device = ScannedDevice {
            name: "/dev/sda".into(),
            device_type: None,
            protocol: Some("ATA".into()),
        };
        let disk = parse_smart_report(&device, json, Some(0), 90);
        assert_eq!(disk.endurance_used_percent, None);
        assert_eq!(disk.remaining_endurance_percent, None);
    }

    #[test]
    fn nvme_used_percentage_falls_back_to_legacy_health_log() {
        let device = ScannedDevice {
            name: "/dev/nvme0".into(),
            device_type: Some("nvme".into()),
            protocol: Some("NVMe".into()),
        };
        let json = r#"{"smart_status":{"passed":true},"nvme_smart_health_information_log":{"percentage_used":12}}"#;
        let disk = parse_smart_report(&device, json, Some(0), 90);
        assert_eq!(disk.endurance_used_percent, Some(12.0));
        assert_eq!(disk.remaining_endurance_percent, Some(88.0));
        assert_eq!(disk.status, "attention");
        assert!(disk
            .findings
            .iter()
            .any(|finding| finding.contains("88.0%")));
    }

    #[test]
    fn nvme_json_report_is_normalized() {
        let device = ScannedDevice {
            name: "/dev/nvme0".into(),
            device_type: Some("nvme".into()),
            protocol: Some("NVMe".into()),
        };
        let json = r#"{
            "model_name":"Example NVMe",
            "user_capacity":{"bytes":1000000000000},
            "endurance_used":{"current_percent":7},
            "spare_available":{"current_percent":100,"threshold_percent":10},
            "power_on_time":{"hours":1200},
            "smart_status":{"passed":true},
            "temperature":{"current":38},
            "nvme_smart_health_information_log":{
                "critical_warning":0,
                "available_spare":100,
                "available_spare_threshold":10,
                "percentage_used":7,
                "media_errors":0,
                "power_on_hours":1200
            }
        }"#;
        let disk = parse_smart_report(&device, json, Some(0), 90);
        assert_eq!(disk.model.as_deref(), Some("Example NVMe"));
        assert_eq!(disk.remaining_endurance_percent, Some(93.0));
        assert_eq!(disk.status, "healthy");
        assert_eq!(disk.temperature_c, Some(38.0));
    }

    #[test]
    fn scan_json_with_empty_device_list_does_not_parse_json_as_text_paths() {
        let devices = parse_scanned_devices(r#"{"devices":[]}"#);
        assert!(devices.is_empty());
    }

    #[test]
    fn scan_json_extracts_device_and_recommended_type() {
        let devices = parse_scanned_devices(
            r#"{"devices":[{"name":"/dev/sda","type":"sat","protocol":"ATA"}]}"#,
        );
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "/dev/sda");
        assert_eq!(devices[0].device_type.as_deref(), Some("sat"));
        assert_eq!(devices[0].protocol.as_deref(), Some("ATA"));
    }

    #[test]
    fn scan_text_fallback_extracts_device_and_type() {
        let devices = parse_scanned_devices("/dev/sda -d sat # ATA disk\n");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "/dev/sda");
        assert_eq!(devices[0].device_type.as_deref(), Some("sat"));
    }

    #[test]
    fn scanned_device_names_reject_option_injection_and_controls() {
        assert!(!valid_device_name("--help"));
        assert!(!valid_device_name("/dev/sda\n--format"));
        assert!(!valid_device_name("/dev/sda\u{0007}"));
        assert!(valid_device_name("/dev/nvme0n1"));
    }

    #[test]
    fn battery_health_requires_both_valid_capacity_values() {
        assert_eq!(
            calculate_battery_health(Some(45_000), Some(50_000)),
            Some(90.0)
        );
        assert_eq!(calculate_battery_health(Some(50_000), Some(0)), None);
        assert_eq!(calculate_battery_health(Some(0), Some(50_000)), None);
        assert_eq!(calculate_battery_health(Some(45_000), None), None);
        assert_eq!(calculate_battery_health(None, Some(50_000)), None);
    }

    #[test]
    fn winget_table_parser_extracts_update_and_keeps_multiword_name() {
        let output = "Name                     Id                           Version  Available  Source\n------------------------  ---------------------------  -------  ---------  ------\nVisual Studio Code        Microsoft.VisualStudioCode   1.90.0   1.91.1     winget\n";
        let updates = parse_winget_upgrade_table(output);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].name, "Visual Studio Code");
        assert_eq!(updates[0].version.as_deref(), Some("1.91.1"));
        assert!(updates[0]
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("Microsoft.VisualStudioCode"));
    }

    #[test]
    fn winget_table_parser_does_not_invent_rows_when_no_table_exists() {
        let output = "No installed package found matching input criteria.\n";
        assert!(parse_winget_upgrade_table(output).is_empty());
    }

    #[test]
    fn zypper_table_parser_extracts_name_available_version_and_repository() {
        let output = "S | Repository | Name | Current Version | Available Version | Arch\n--+------------+------+-----------------+-------------------+------\nv | Main Repo  | example-package | 1.0 | 1.1 | x86_64\n";
        let updates = parse_zypper_update_table(output);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].name, "example-package");
        assert_eq!(updates[0].version.as_deref(), Some("1.1"));
        assert!(updates[0]
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("Main Repo"));
    }

    #[test]
    fn apt_simulation_parser_uses_available_version_not_installed_version() {
        let output =
            "Inst openssl [3.0.2-0] (3.0.2-1 security [amd64])\nInst curl (8.0.0-1 main [amd64])\n";
        let updates = parse_apt_simulation(output);
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].name, "openssl");
        assert_eq!(updates[0].version.as_deref(), Some("3.0.2-1"));
        assert_eq!(updates[1].version.as_deref(), Some("8.0.0-1"));
    }

    #[test]
    fn apt_parser_counts_all_rows_before_ui_display_limit() {
        let output = (0..125)
            .map(|index| format!("Inst package-{index} (1.0.{index} repo [amd64])"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_apt_simulation(&output).len(), 125);
    }

    #[test]
    fn dnf_parser_skips_metadata_lines() {
        let output = "Updating and loading repositories:\nLast metadata expiration check: 0:01:20 ago\nopenssl.x86_64  3.2.1-1.fc41  updates\ncurl.x86_64  8.10.0-2.fc41  updates\n";
        let updates = parse_dnf_check_update(output);
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].name, "openssl.x86_64");
        assert_eq!(updates[0].version.as_deref(), Some("3.2.1-1.fc41"));
    }

    #[test]
    fn pacman_parser_requires_old_and_new_versions() {
        let output = "curl 8.9.0-1 -> 8.10.0-1\nopenssl 3.3.1-1 -> 3.3.2-1\nnot-a-package-line\n";
        let updates = parse_pacman_upgrade_list(output);
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].name, "curl");
        assert_eq!(updates[0].version.as_deref(), Some("8.10.0-1"));
        assert!(updates[0]
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("8.9.0-1"));
    }

    #[test]
    fn battery_health_does_not_accept_zero_full_capacity() {
        assert_eq!(calculate_battery_health(Some(0), Some(50_000)), None);
    }

    #[test]
    fn nvidia_smi_parser_preserves_unavailable_metrics() {
        let output = "NVIDIA GeForce RTX 4060, 42, 8, 8192, 2048, 61, 2100, 90.5\n";
        let parsed = parse_nvidia_smi_csv(output);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "NVIDIA GeForce RTX 4060");
        assert_eq!(parsed[0].utilization_percent, Some(42.0));
        assert_eq!(parsed[0].temperature_c, Some(61.0));
        assert_eq!(parsed[0].memory_total_mib, Some(8192.0));
        assert_eq!(parsed[0].power_draw_watts, Some(90.5));
    }

    #[test]
    fn nvidia_smi_parser_does_not_invent_na_values() {
        let output = "NVIDIA GPU, N/A, N/A, 8192, 0, N/A, N/A, N/A\n";
        let parsed = parse_nvidia_smi_csv(output);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].utilization_percent, None);
        assert_eq!(parsed[0].temperature_c, None);
        assert_eq!(parsed[0].power_draw_watts, None);
    }

    #[test]
    fn vendor_support_profile_maps_common_pc_brands_and_families() {
        let dell = vendor_support_profile("Dell Inc.", "Latitude 5420");
        assert_eq!(dell.vendor, "Dell");
        assert_eq!(dell.product_family.as_deref(), Some("latitude"));
        assert!(dell
            .support_url
            .as_deref()
            .unwrap_or_default()
            .contains("dell.com"));
        let lenovo = vendor_support_profile("LENOVO", "ThinkPad T14");
        assert_eq!(lenovo.product_family.as_deref(), Some("thinkpad"));
        let framework = vendor_support_profile("Framework Computer Inc.", "Laptop 13");
        assert_eq!(framework.vendor, "Framework");
        assert!(framework
            .support_url
            .as_deref()
            .unwrap_or_default()
            .contains("frame.work"));
    }

    #[test]
    fn unknown_vendor_profile_does_not_guess_a_support_url() {
        let profile = vendor_support_profile("Unknown OEM", "Custom Build");
        assert_eq!(profile.vendor, "Unknown / custom manufacturer");
        assert_eq!(profile.support_url, None);
    }

    #[test]
    fn pnp_problem_codes_have_specific_cautious_explanations() {
        assert!(pnp_problem_description(28)
            .to_ascii_lowercase()
            .contains("not installed"));
        assert!(pnp_problem_description(43)
            .to_ascii_lowercase()
            .contains("stopped"));
        assert!(pnp_problem_description(999).contains("Configuration Manager"));
    }

    #[test]
    fn driver_findings_include_windows_problem_code_and_safe_recommendation() {
        let problems = vec![ProblemDevice {
            name: "Example USB device".into(),
            class: Some("USB".into()),
            status: "Configuration Manager error 28".into(),
            instance_id: Some("USB\\VID_0000&PID_0000".into()),
            problem_code: Some(28),
            problem_description: Some(pnp_problem_description(28).into()),
            source: "fixture".into(),
        }];
        let findings = collect_driver_findings(&[], &problems);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "PNP_CONFIG_MANAGER_28");
        assert!(findings[0].recommendation.contains("hardware ID"));
    }

    #[test]
    fn driver_findings_flag_only_evidence_based_issues() {
        let drivers = vec![DriverInfo {
            device: "Example Device".into(),
            manufacturer: None,
            driver: Some("example.sys".into()),
            version: Some("1.0".into()),
            signed: Some(false),
            source: "test".into(),
        }];
        let findings = collect_driver_findings(&drivers, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "DRIVER_SIGNATURE_NOT_CONFIRMED");
        let normal = vec![DriverInfo {
            device: "Normal Device".into(),
            manufacturer: None,
            driver: Some("normal.sys".into()),
            version: Some("1.0".into()),
            signed: Some(true),
            source: "test".into(),
        }];
        assert!(collect_driver_findings(&normal, &[]).is_empty());
    }

    #[test]
    fn amd_smi_parser_reads_supported_metrics_and_keeps_unknowns_missing() {
        let sample = r#"{"GPU":{"0":{"ASIC":{"MARKET_NAME":"AMD Radeon Test"},"METRIC":{"GFX_ACTIVITY":"42 %","MEM_ACTIVITY":"18 %","HOTSPOT":"67 °C","SOCKET_POWER":"91 W","GFX_CLOCK":"2100 MHz"}},"1":{"METRIC":{"GFX_ACTIVITY":"N/A"}}}}"#;
        let parsed = parse_amd_smi_json(sample);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "AMD Radeon Test");
        assert_eq!(parsed[0].utilization_percent, Some(42.0));
        assert_eq!(parsed[0].memory_utilization_percent, Some(18.0));
        assert_eq!(parsed[0].temperature_c, Some(67.0));
        assert_eq!(parsed[0].power_draw_watts, Some(91.0));
        assert_eq!(parsed[0].graphics_clock_mhz, Some(2100.0));
    }

    #[test]
    fn intel_gpu_top_parser_uses_engine_busy_and_ignores_dash_values() {
        let sample = r#"{"device":{"name":"Intel Test Graphics","engines":{"Render/3D":{"busy":31.5},"Video":{"busy":"N/A"},"Blitter":{"busy":12.0}}}}"#;
        let parsed = parse_intel_gpu_top_json(sample);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "Intel Test Graphics");
        assert_eq!(parsed[0].utilization_percent, Some(31.5));
        assert_eq!(parsed[0].temperature_c, None);
    }

    #[test]
    fn lspci_parser_reports_bound_driver_and_unbound_module_candidate_separately() {
        let sample = "0000:00:02.0 VGA compatible controller: Intel Test Graphics [8086:1234]\n\tKernel driver in use: i915\n\tKernel modules: i915\n0000:03:00.0 Network controller: Example Wi-Fi [1234:5678]\n\tKernel modules: example_wifi\n";
        let parsed = parse_lspci_driver_output(sample);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].driver.as_deref(), Some("i915"));
        assert!(parsed[1]
            .driver
            .as_deref()
            .unwrap_or_default()
            .starts_with("Not bound; candidate kernel modules:"));
    }

    #[test]
    fn sha256_input_requires_exact_hex_digest() {
        assert!(validate_sha256_input(Some("a".repeat(64))).is_ok());
        assert!(validate_sha256_input(Some("a".repeat(63))).is_err());
        assert!(validate_sha256_input(Some("g".repeat(64))).is_err());
        assert_eq!(validate_sha256_input(None).unwrap(), None);
    }

    #[test]
    fn recovery_image_hash_matches_known_sha256_fixture() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!(
            "crossdiag-sha256-{}-{}.iso",
            std::process::id(),
            timestamp_string().replace(':', "-").replace(' ', "-")
        ));
        let mut file = fs::File::create(&path).expect("create hash test fixture");
        file.write_all(b"hello").expect("write hash test fixture");
        drop(file);
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let result = inspect_recovery_image_blocking(path.to_str().unwrap(), Some(expected.into()))
            .expect("inspect hash test fixture");
        assert_eq!(result.sha256, expected);
        assert_eq!(result.checksum_matches, Some(true));
        let mismatch =
            inspect_recovery_image_blocking(path.to_str().unwrap(), Some("a".repeat(64)))
                .expect("inspect checksum mismatch");
        assert_eq!(mismatch.checksum_matches, Some(false));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn amd_smi_parser_rejects_out_of_range_utilization_and_retains_other_metrics() {
        let sample = r#"{"GPU":{"0":{"ASIC":{"MARKET_NAME":"AMD Example"},"METRIC":{"GFX_ACTIVITY":"105 %","HOTSPOT":"70 C"}}}}"#;
        let parsed = parse_amd_smi_json(sample);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].utilization_percent, None);
        assert_eq!(parsed[0].temperature_c, Some(70.0));
    }

    #[test]
    fn lsblk_parser_only_returns_removable_disk_devices_and_marks_mounts() {
        let sample = r#"{"blockdevices":[{"name":"nvme0n1","size":512000,"rm":false,"type":"disk","tran":"nvme","mountpoints":[null]},{"name":"sdb","size":32000000000,"rm":true,"type":"disk","model":"USB Stick","tran":"usb","mountpoints":[null],"children":[{"name":"sdb1","mountpoints":["/media/user/USB"]}]},{"name":"sdc","size":64000000000,"rm":false,"type":"disk","model":"USB SSD","tran":"usb","mountpoints":["/"]}]}"#;
        let targets = parse_lsblk_recovery_targets(sample);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].path, "/dev/sdb");
        assert!(targets[0].mounted);
        assert_eq!(targets[0].system_disk, Some(false));
        assert!(targets[1].system_disk == Some(true));
    }

    #[test]
    fn scheduled_storage_scan_honors_interval_and_disabled_state() {
        assert!(!scheduled_scan_is_due(false, 1, None, 10_000));
        assert!(scheduled_scan_is_due(true, 24, None, 10_000));
        assert!(!scheduled_scan_is_due(
            true,
            24,
            Some(10_000),
            10_000 + 86_399
        ));
        assert!(scheduled_scan_is_due(
            true,
            24,
            Some(10_000),
            10_000 + 86_400
        ));
    }

    #[test]
    fn scheduled_scan_interval_is_clamped_to_one_through_168_hours() {
        assert!(!scheduled_scan_is_due(true, 0, Some(10), 10 + 3_599));
        assert!(scheduled_scan_is_due(true, 0, Some(10), 10 + 3_600));
        assert!(!scheduled_scan_is_due(true, 999, Some(10), 10 + 604_799));
    }

    #[test]
    fn memory_patterns_differ_between_passes() {
        for pass in 0..6 {
            for other in (pass + 1)..6 {
                assert_ne!(
                    memory_test_pattern_for_pass(5, pass),
                    memory_test_pattern_for_pass(5, other)
                );
            }
        }
        assert_eq!(
            memory_test_pattern_for_pass(5, 0),
            memory_test_pattern_for_pass(5, 6)
        );
    }
}
