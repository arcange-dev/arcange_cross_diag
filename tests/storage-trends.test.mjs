import test from "node:test";
import assert from "node:assert/strict";
import { compareStorageScans, storageScansToCsv, formatMetricDelta, isMetricWorsening } from "../src/lib/storageTrends.mjs";

const oldScan = {
  scanned_at: "2026-11-08T10:00:00.000Z",
  threshold_percent: 90,
  disks: [{
    device: "/dev/nvme0n1", model: "Example NVMe", status: "healthy",
    remaining_endurance_percent: 96, endurance_used_percent: 4, temperature_c: 40,
    available_spare_percent: 100, media_errors: 0, power_on_hours: 100,
    findings: [], limitations: [],
  }],
};
const newScan = {
  scanned_at: "2026-11-09T10:00:00.000Z",
  threshold_percent: 90,
  disks: [{
    device: "/dev/nvme0n1", model: "Example NVMe", status: "attention",
    remaining_endurance_percent: 94, endurance_used_percent: 6, temperature_c: 45,
    available_spare_percent: 99, media_errors: 0, power_on_hours: 124,
    findings: ["Temperature increased"], limitations: [],
  }],
};

test("compares the two newest scans and reports meaningful deltas", () => {
  const result = compareStorageScans([newScan, oldScan]);
  assert.ok(result);
  assert.equal(result.previousScannedAt, oldScan.scanned_at);
  assert.equal(result.currentScannedAt, newScan.scanned_at);
  assert.equal(result.changes[0].change, "changed");
  assert.equal(result.changes[0].statusChanged, true);
  assert.equal(result.changes[0].metrics.find((metric) => metric.metric === "remaining_endurance_percent").delta, -2);
  assert.equal(result.changes[0].metrics.find((metric) => metric.metric === "temperature_c").delta, 5);
});

test("does not interpret missing readings as zero", () => {
  const result = compareStorageScans([
    { ...oldScan, disks: [{ ...oldScan.disks[0], temperature_c: null }] },
    newScan,
  ]);
  assert.ok(result);
  assert.equal(result.changes[0].metrics.some((metric) => metric.metric === "temperature_c"), false);
});

test("reports added and removed devices separately", () => {
  const before = { ...oldScan, disks: [{ ...oldScan.disks[0], device: "/dev/sda" }] };
  const after = { ...newScan, disks: [{ ...newScan.disks[0], device: "/dev/nvme0n1" }] };
  const result = compareStorageScans([before, after]);
  assert.deepEqual(result.changes.map((item) => item.change).sort(), ["added", "removed"]);
});

test("returns null until two valid timestamped scans exist", () => {
  assert.equal(compareStorageScans([]), null);
  assert.equal(compareStorageScans([oldScan]), null);
  assert.equal(compareStorageScans([{ ...oldScan, scanned_at: "not-a-date" }, newScan]), null);
});

test("exports CSV with quoted commas and keeps missing metrics empty", () => {
  const csv = storageScansToCsv([{ ...oldScan, disks: [{ ...oldScan.disks[0], model: "Disk, revision A", temperature_c: null, findings: ["Needs attention", "Check, cable"] }] }]);
  assert.match(csv, /"Disk, revision A"/);
  assert.match(csv, /"Needs attention \| Check, cable"/);
  assert.match(csv, /remaining_endurance_percent/);
  assert.doesNotMatch(csv, /serial_number|serial number/i);
  const formulaSafe = storageScansToCsv([{ ...oldScan, disks: [{ ...oldScan.disks[0], model: "=HYPERLINK(\"https://example.invalid\")" }] }]);
  assert.match(formulaSafe, /'=?HYPERLINK/);
});

test("formats metric deltas with a sign and unit", () => {
  assert.equal(formatMetricDelta({ delta: -1.5, unit: "%" }), "-1.5%");
  assert.equal(formatMetricDelta({ delta: 3, unit: "h" }), "+3.0h");
  assert.equal(formatMetricDelta({ delta: 4, unit: "" }), "+4");
});


test("only interprets directional health metrics as worsening", () => {
  assert.equal(isMetricWorsening("remaining_endurance_percent", -1), true);
  assert.equal(isMetricWorsening("remaining_endurance_percent", 1), false);
  assert.equal(isMetricWorsening("available_spare_percent", -1), true);
  assert.equal(isMetricWorsening("media_errors", 1), true);
  assert.equal(isMetricWorsening("temperature_c", 1), true);
  assert.equal(isMetricWorsening("power_on_hours", 50), false);
  assert.equal(isMetricWorsening("power_on_hours", -1), false);
});


test("orders unix-prefixed Rust timestamps correctly", () => {
  const unixOlder = { ...oldScan, scanned_at: "unix:1794128400" };
  const unixNewer = { ...newScan, scanned_at: "unix:1794214800" };
  const result = compareStorageScans([unixNewer, unixOlder]);
  assert.ok(result);
  assert.equal(result.previousScannedAt, unixOlder.scanned_at);
  assert.equal(result.currentScannedAt, unixNewer.scanned_at);
});
