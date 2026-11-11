const METRICS = [
  ["remaining_endurance_percent", "Estimated remaining endurance", "%"],
  ["endurance_used_percent", "Endurance used", "%"],
  ["temperature_c", "Temperature", "°C"],
  ["available_spare_percent", "Available spare", "%"],
  ["media_errors", "Media errors", ""],
  ["reallocated_sectors", "Reallocated sectors", ""],
  ["current_pending_sectors", "Pending sectors", ""],
  ["offline_uncorrectable", "Uncorrectable sectors", ""],
  ["power_on_hours", "Power-on hours", "h"],
];

function timestampOf(value) {
  if (typeof value !== "string" || !value.trim()) return Number.NaN;
  if (value.startsWith("unix:")) {
    const seconds = Number(value.slice(5));
    return Number.isFinite(seconds) ? seconds * 1000 : Number.NaN;
  }
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : Number.NaN;
}

function finiteMetric(value) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function csvCell(value) {
  if (value === null || value === undefined) return "";
  let text = Array.isArray(value) ? value.join(" | ") : String(value);
  // Protect text fields from spreadsheet formula execution. Numeric cells remain numeric.
  if (typeof value !== "number" && /^[\t\r ]*[=+\-@]/.test(text)) text = `'${text}`;
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

/**
 * Compare the two newest valid scans. Device paths are used as local identifiers;
 * serial numbers are deliberately neither collected nor exported here.
 */
export function compareStorageScans(scans) {
  if (!Array.isArray(scans)) return null;
  const ordered = scans
    .filter((scan) => scan && typeof scan.scanned_at === "string" && Array.isArray(scan.disks))
    .map((scan) => ({ scan, timestamp: timestampOf(scan.scanned_at) }))
    .filter((item) => Number.isFinite(item.timestamp))
    .sort((a, b) => a.timestamp - b.timestamp);

  if (ordered.length < 2) return null;
  const previous = ordered[ordered.length - 2].scan;
  const current = ordered[ordered.length - 1].scan;
  const before = new Map(previous.disks.filter((disk) => disk && typeof disk.device === "string").map((disk) => [disk.device.toLowerCase(), disk]));
  const after = new Map(current.disks.filter((disk) => disk && typeof disk.device === "string").map((disk) => [disk.device.toLowerCase(), disk]));
  const keys = new Set([...before.keys(), ...after.keys()]);
  const changes = [];

  for (const key of keys) {
    const oldDisk = before.get(key) ?? null;
    const newDisk = after.get(key) ?? null;
    if (!oldDisk) {
      changes.push({ device: newDisk.device, model: newDisk.model ?? null, change: "added", previousStatus: null, currentStatus: newDisk.status ?? "unknown", metrics: [] });
      continue;
    }
    if (!newDisk) {
      changes.push({ device: oldDisk.device, model: oldDisk.model ?? null, change: "removed", previousStatus: oldDisk.status ?? "unknown", currentStatus: null, metrics: [] });
      continue;
    }

    const metrics = [];
    for (const [metric, label, unit] of METRICS) {
      const previousValue = finiteMetric(oldDisk[metric]);
      const currentValue = finiteMetric(newDisk[metric]);
      // Missing readings are unknown, never silently converted to zero.
      if (previousValue === null || currentValue === null || previousValue === currentValue) continue;
      metrics.push({ metric, label, unit, previous: previousValue, current: currentValue, delta: currentValue - previousValue });
    }
    const statusChanged = (oldDisk.status ?? "unknown") !== (newDisk.status ?? "unknown");
    changes.push({
      device: newDisk.device,
      model: newDisk.model ?? oldDisk.model ?? null,
      change: statusChanged || metrics.length ? "changed" : "unchanged",
      previousStatus: oldDisk.status ?? "unknown",
      currentStatus: newDisk.status ?? "unknown",
      statusChanged,
      metrics,
    });
  }

  return {
    previousScannedAt: previous.scanned_at,
    currentScannedAt: current.scanned_at,
    previousThresholdPercent: previous.threshold_percent,
    currentThresholdPercent: current.threshold_percent,
    changes,
  };
}

/** Convert saved SMART scans to CSV, retaining null readings as empty cells. */
export function storageScansToCsv(scans) {
  const headers = [
    "scanned_at", "threshold_percent", "device", "model", "device_type", "protocol", "drive_kind", "status", "status_reason",
    "smart_passed", "remaining_endurance_percent", "endurance_used_percent", "endurance_basis", "temperature_c", "available_spare_percent",
    "available_spare_threshold_percent", "critical_warning", "media_errors", "reallocated_sectors", "current_pending_sectors",
    "offline_uncorrectable", "power_on_hours", "capacity_bytes", "smartctl_exit_code", "findings", "limitations",
  ];
  const rows = [headers.join(",")];
  for (const scan of Array.isArray(scans) ? scans : []) {
    for (const disk of Array.isArray(scan?.disks) ? scan.disks : []) {
      const row = [
        scan.scanned_at, scan.threshold_percent, disk.device, disk.model, disk.device_type, disk.protocol, disk.drive_kind, disk.status, disk.status_reason,
        disk.smart_passed, disk.remaining_endurance_percent, disk.endurance_used_percent, disk.endurance_basis, disk.temperature_c, disk.available_spare_percent,
        disk.available_spare_threshold_percent, disk.critical_warning, disk.media_errors, disk.reallocated_sectors, disk.current_pending_sectors,
        disk.offline_uncorrectable, disk.power_on_hours, disk.capacity_bytes, disk.smartctl_exit_code, disk.findings, disk.limitations,
      ];
      rows.push(row.map(csvCell).join(","));
    }
  }
  return `${rows.join("\r\n")}\r\n`;
}

export function isMetricWorsening(metricName, delta) {
  if (!Number.isFinite(delta) || delta === 0) return false;
  const lowerIsWorse = new Set(["remaining_endurance_percent", "available_spare_percent"]);
  const higherIsWorse = new Set([
    "endurance_used_percent", "temperature_c", "media_errors", "reallocated_sectors",
    "current_pending_sectors", "offline_uncorrectable",
  ]);
  if (lowerIsWorse.has(metricName)) return delta < 0;
  if (higherIsWorse.has(metricName)) return delta > 0;
  // Power-on hours naturally increase; a trend alone is not a fault.
  return false;
}

export function formatMetricDelta(metric) {
  if (!metric || !Number.isFinite(metric.delta)) return "No comparable measurement";
  const precision = metric.unit === "" ? 0 : 1;
  const sign = metric.delta > 0 ? "+" : "";
  return `${sign}${metric.delta.toFixed(precision)}${metric.unit}`;
}
