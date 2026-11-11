import { invoke, isTauri } from "@tauri-apps/api/core";

export type DriveInfo = {
  name: string;
  mount_point: string;
  file_system: string;
  total_bytes: number;
  available_bytes: number;
  removable: boolean;
};

export type SystemSnapshot = {
  app_version: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  hostname: string;
  manufacturer: string;
  model: string;
  architecture: string;
  cpu_name: string;
  logical_cores: number;
  physical_cores: number | null;
  total_memory_bytes: number;
  used_memory_bytes: number;
  drives: DriveInfo[];
  collected_at: string;
  capabilities: string[];
  limitations: string[];
};

export type SmartAttribute = {
  id: number | null;
  name: string;
  normalized_value: number | null;
  worst_value: number | null;
  threshold: number | null;
  raw_value: number | null;
  when_failed: string | null;
};

export type PhysicalDiskHealth = {
  device: string;
  device_type: string | null;
  protocol: string | null;
  model: string | null;
  drive_kind: string;
  capacity_bytes: number | null;
  rotation_rate_rpm: number | null;
  smart_passed: boolean | null;
  status: "healthy" | "attention" | "failed" | "unknown" | string;
  status_reason: string;
  findings: string[];
  remaining_endurance_percent: number | null;
  endurance_used_percent: number | null;
  endurance_basis: string | null;
  temperature_c: number | null;
  available_spare_percent: number | null;
  available_spare_threshold_percent: number | null;
  critical_warning: number | null;
  media_errors: number | null;
  reallocated_sectors: number | null;
  current_pending_sectors: number | null;
  offline_uncorrectable: number | null;
  power_on_hours: number | null;
  smart_attributes: SmartAttribute[];
  smartctl_exit_code: number | null;
  limitations: string[];
};

export type StorageScan = {
  scanned_at: string;
  threshold_percent: number;
  smartctl_available: boolean;
  smartctl_version: string | null;
  scan_status: string;
  disks: PhysicalDiskHealth[];
  limitations: string[];
};

function requireDesktop(): void {
  if (!isTauri()) {
    throw new Error("This diagnostic requires the CrossDiag desktop application. Start it with `npm run tauri dev`.");
  }
}

export async function getSystemSnapshot(): Promise<SystemSnapshot> {
  requireDesktop();
  return invoke<SystemSnapshot>("get_system_snapshot");
}

export type RuntimeToolCheck = {
  id: string;
  label: string;
  category: string;
  available: boolean;
  purpose: string;
  install_hint: string;
};

export type RuntimeReadiness = {
  app_version: string;
  platform: string;
  architecture: string;
  checked_at: string;
  tools: RuntimeToolCheck[];
  limitations: string[];
};

export async function getRuntimeReadiness(): Promise<RuntimeReadiness> {
  requireDesktop();
  return invoke<RuntimeReadiness>("get_runtime_readiness");
}

export async function scanStorageHealth(thresholdPercent: number): Promise<StorageScan> {
  requireDesktop();
  const safeThreshold = Math.max(0, Math.min(100, Math.round(thresholdPercent)));
  return invoke<StorageScan>("scan_storage_health", { thresholdPercent: safeThreshold });
}

export async function getStorageHistory(): Promise<StorageScan[]> {
  requireDesktop();
  return invoke<StorageScan[]>("get_storage_history");
}

export type HardwareSensor = {
  name: string;
  category: string;
  value: number;
  unit: string;
  status: string;
  critical_limit: number | null;
  source: string;
};

export type GpuDevice = {
  name: string;
  vendor_id: string | null;
  device_id: string | null;
  driver: string | null;
  status: string;
  vram_bytes: number | null;
  source: string;
};

export type BatteryInfo = {
  name: string;
  status: string | null;
  charge_percent: number | null;
  health_percent: number | null;
  design_capacity_mwh: number | null;
  full_charge_capacity_mwh: number | null;
  source: string;
  limitations: string[];
};

export type NetworkAdapterInfo = {
  name: string;
  status: string | null;
  link_speed: string | null;
  mac_address: string | null;
  received_bytes: number;
  transmitted_bytes: number;
  source: string;
};

export type ProblemDevice = {
  name: string;
  class: string | null;
  status: string;
  instance_id: string | null;
  problem_code: number | null;
  problem_description: string | null;
  source: string;
};

export type FirmwareInfo = {
  vendor: string | null;
  version: string | null;
  release_date: string | null;
  mode: string | null;
  secure_boot: boolean | null;
  tpm_detected: boolean | null;
  source: string;
};

export type MemoryModule = {
  locator: string | null;
  manufacturer: string | null;
  part_number: string | null;
  capacity_bytes: number | null;
  speed_mhz: number | null;
  configured_speed_mhz: number | null;
  source: string;
};

export type DriverInfo = {
  device: string;
  manufacturer: string | null;
  driver: string | null;
  version: string | null;
  signed: boolean | null;
  source: string;
};

export type PeripheralInfo = {
  category: string;
  name: string;
  status: string | null;
  identifier: string | null;
  link_speed: string | null;
  source: string;
};

export type CpuCoreInfo = { name: string; usage_percent: number; frequency_mhz: number | null };
export type GpuTelemetry = {
  name: string;
  utilization_percent: number | null;
  memory_utilization_percent: number | null;
  memory_total_mib: number | null;
  memory_used_mib: number | null;
  temperature_c: number | null;
  graphics_clock_mhz: number | null;
  power_draw_watts: number | null;
  source: string;
};
export type VendorSupportProfile = {
  vendor: string;
  product_family: string | null;
  support_url: string | null;
  diagnostic_utility: string | null;
  firmware_setup_hint: string;
  boot_menu_hint: string;
  notes: string;
};
export type DriverFinding = {
  device: string;
  severity: string;
  code: string;
  explanation: string;
  recommendation: string;
};

export type HardwareDiagnostics = {
  scanned_at: string;
  cpu_usage_percent: number;
  cpu_frequency_mhz: number | null;
  cpu_cores: CpuCoreInfo[];
  available_memory_bytes: number;
  memory_modules: MemoryModule[];
  drivers: DriverInfo[];
  driver_findings: DriverFinding[];
  peripherals: PeripheralInfo[];
  gpus: GpuDevice[];
  gpu_telemetry: GpuTelemetry[];
  sensors: HardwareSensor[];
  batteries: BatteryInfo[];
  network_adapters: NetworkAdapterInfo[];
  problem_devices: ProblemDevice[];
  firmware: FirmwareInfo;
  vendor_support: VendorSupportProfile;
  limitations: string[];
};

export type AvailableUpdate = {
  name: string;
  version: string | null;
  source: string;
  detail: string | null;
  install_id: string | null;
};

export type UpdateStatus = {
  checked_at: string;
  platform: string;
  package_manager: string;
  status: string;
  update_count: number | null;
  updates: AvailableUpdate[];
  limitations: string[];
};

export type PerformanceTestResult = {
  test_id: string;
  status: string;
  started_at: string;
  completed_at: string;
  duration_ms: number;
  metric_value: number | null;
  metric_unit: string;
  summary: string;
  limitations: string[];
};

export type SystemFault = {
  timestamp: string | null;
  severity: string;
  source: string;
  code: string | null;
  message: string;
};

export type FaultReport = {
  scanned_at: string;
  platform: string;
  source: string;
  faults: SystemFault[];
  limitations: string[];
};

export async function getHardwareDiagnostics(): Promise<HardwareDiagnostics> {
  requireDesktop();
  return invoke<HardwareDiagnostics>("get_hardware_diagnostics");
}

export async function runPerformanceTest(testId: "cpu_single_core" | "cpu_multi_core" | "memory_smoke", durationSeconds = 5, memoryMb = 64): Promise<PerformanceTestResult> {
  requireDesktop();
  return invoke<PerformanceTestResult>("run_performance_test", { testId, durationSeconds, memoryMb });
}

export async function checkUpdateStatus(): Promise<UpdateStatus> {
  requireDesktop();
  return invoke<UpdateStatus>("check_update_status");
}

export async function analyzeSystemFaults(): Promise<FaultReport> {
  requireDesktop();
  return invoke<FaultReport>("analyze_system_faults");
}


export type UpdateInstallResult = {
  package: string;
  source: string;
  started_at: string;
  completed_at: string;
  success: boolean;
  exit_code: number | null;
  output: string;
  limitations: string[];
};

export type RepairAction = {
  id: string;
  title: string;
  description: string;
  category: string;
  is_read_only: boolean;
  requires_admin: boolean;
};

export type RepairResult = {
  action_id: string;
  title: string;
  started_at: string;
  completed_at: string;
  success: boolean;
  exit_code: number | null;
  output: string;
  limitations: string[];
};

export type TechnicianCase = {
  id: number;
  title: string;
  device: string;
  status: "open" | "in_progress" | "resolved" | "archived" | string;
  notes: string;
  created_at: string;
  updated_at: string;
};

export type ScanSchedule = {
  enabled: boolean;
  interval_hours: number;
  threshold_percent: number;
};

export async function installSelectedUpdate(packageId: string, source: string): Promise<UpdateInstallResult> {
  requireDesktop();
  return invoke<UpdateInstallResult>("install_selected_update", { package: packageId, source });
}

export async function getRepairCatalog(): Promise<RepairAction[]> {
  requireDesktop();
  return invoke<RepairAction[]>("get_repair_catalog");
}

export async function runRepairAction(actionId: string): Promise<RepairResult> {
  requireDesktop();
  return invoke<RepairResult>("run_repair_action", { actionId });
}

export async function listTechnicianCases(): Promise<TechnicianCase[]> {
  requireDesktop();
  return invoke<TechnicianCase[]>("list_technician_cases");
}

export async function createTechnicianCase(title: string, device: string, notes: string): Promise<TechnicianCase> {
  requireDesktop();
  return invoke<TechnicianCase>("create_technician_case", { title, device, notes });
}

export async function updateTechnicianCase(id: number, status: string, notes: string): Promise<void> {
  requireDesktop();
  return invoke<void>("update_technician_case", { id, status, notes });
}

export async function deleteTechnicianCase(id: number): Promise<void> {
  requireDesktop();
  return invoke<void>("delete_technician_case", { id });
}

export async function getScanSchedule(): Promise<ScanSchedule> {
  requireDesktop();
  return invoke<ScanSchedule>("get_scan_schedule");
}

export async function saveScanSchedule(enabled: boolean, intervalHours: number, thresholdPercent = 90): Promise<ScanSchedule> {
  requireDesktop();
  const safeThreshold = Math.max(0, Math.min(100, Math.round(thresholdPercent)));
  return invoke<ScanSchedule>("save_scan_schedule", { enabled, intervalHours, thresholdPercent: safeThreshold });
}

export async function openDriverManager(): Promise<string> {
  requireDesktop();
  return invoke<string>("open_driver_manager");
}

export async function openSystemUpdateSettings(): Promise<string> {
  requireDesktop();
  return invoke<string>("open_system_update_settings");
}

export async function installOperatingSystemUpdates(): Promise<UpdateInstallResult> {
  requireDesktop();
  return invoke<UpdateInstallResult>("install_operating_system_updates");
}

export type RecoveryImageInfo = {
  path: string;
  file_name: string;
  size_bytes: number;
  sha256: string;
  expected_sha256: string | null;
  checksum_matches: boolean | null;
  inspected_at: string;
  limitations: string[];
};

export type RecoveryMediaTarget = {
  path: string;
  name: string;
  size_bytes: number;
  transport: string | null;
  removable: boolean;
  system_disk: boolean | null;
  boot_disk: boolean | null;
  mounted: boolean;
  source: string;
};

export async function inspectRecoveryImage(path: string, expectedSha256?: string): Promise<RecoveryImageInfo> {
  requireDesktop();
  return invoke<RecoveryImageInfo>("inspect_recovery_image", { path, expectedSha256: expectedSha256?.trim() || null });
}

export async function listRecoveryMedia(): Promise<RecoveryMediaTarget[]> {
  requireDesktop();
  return invoke<RecoveryMediaTarget[]>("list_recovery_media");
}
