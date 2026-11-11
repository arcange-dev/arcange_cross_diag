import type { StorageScan } from "./desktop";

type StorageMetricDelta = {
  metric: string;
  label: string;
  unit: string;
  previous: number;
  current: number;
  delta: number;
};

type StorageDiskTrend = {
  device: string;
  model: string | null;
  change: "added" | "removed" | "changed" | "unchanged";
  previousStatus: string | null;
  currentStatus: string | null;
  statusChanged?: boolean;
  metrics: StorageMetricDelta[];
};

export type StorageHistoryComparison = {
  previousScannedAt: string;
  currentScannedAt: string;
  previousThresholdPercent: number;
  currentThresholdPercent: number;
  changes: StorageDiskTrend[];
};

export function compareStorageScans(scans: readonly StorageScan[]): StorageHistoryComparison | null;
export function storageScansToCsv(scans: readonly StorageScan[]): string;
export function formatMetricDelta(metric: StorageMetricDelta): string;
export function isMetricWorsening(metricName: string, delta: number): boolean;
