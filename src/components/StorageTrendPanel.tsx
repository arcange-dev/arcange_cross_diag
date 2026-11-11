import { useMemo } from "react";
import { Download, History, Minus, Plus, TrendingDown, TrendingUp } from "lucide-react";
import { compareStorageScans, formatMetricDelta, isMetricWorsening, storageScansToCsv } from "../lib/storageTrends.mjs";
import type { StorageScan } from "../lib/desktop";

type Props = { scans: StorageScan[] };

function dateLabel(value: string): string {
  const normalized = value.startsWith("unix:") ? Number(value.slice(5)) * 1000 : value;
  const parsed = new Date(normalized);
  return Number.isNaN(parsed.getTime()) ? value : parsed.toLocaleString();
}

function downloadCsv(scans: StorageScan[]) {
  const csv = storageScansToCsv(scans);
  const blob = new Blob(["\uFEFF", csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `crossdiag-storage-history-${new Date().toISOString().slice(0, 10)}.csv`;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1_000);
}

export function StorageTrendPanel({ scans }: Props) {
  const comparison = useMemo(() => compareStorageScans(scans), [scans]);
  const changedCount = comparison?.changes.filter((item) => item.change !== "unchanged").length ?? 0;

  return (
    <section className="panel storage-trend-panel">
      <div className="panel-header">
        <div>
          <h3><History size={16}/> Storage health trends</h3>
          <p>Compare the two newest saved SMART scans. Missing measurements are not treated as zero.</p>
        </div>
        <button className="subtle-button" onClick={() => downloadCsv(scans)} disabled={scans.length === 0}>
          <Download size={14}/> Export CSV
        </button>
      </div>
      {!comparison ? (
        <div className="trend-empty">
          <History size={19}/>
          <div><strong>Not enough scan history yet</strong><span>Run at least two SMART scans to compare drive readings over time. This comparison uses saved scan data only.</span></div>
        </div>
      ) : (
        <>
          <div className="trend-comparison-summary">
            <div><span>Previous scan</span><strong>{dateLabel(comparison.previousScannedAt)}</strong></div>
            <div className="trend-arrow"><TrendingUp size={16}/></div>
            <div><span>Latest scan</span><strong>{dateLabel(comparison.currentScannedAt)}</strong></div>
            <div className="trend-count"><strong>{changedCount}</strong><span>drive(s) with changes</span></div>
          </div>
          {comparison.changes.length === 0 ? <p className="trend-footnote">No physical drives were present in either saved scan.</p> : (
            <div className="trend-device-list">
              {comparison.changes.map((item) => (
                <article className="trend-device" key={item.device}>
                  <div className="trend-device-heading">
                    <div><strong>{item.model ?? item.device}</strong><span className="mono">{item.device}</span></div>
                    <span className={`trend-state trend-${item.change}`}>{item.change.replaceAll("_", " ")}</span>
                  </div>
                  {item.change === "added" && <p className="trend-footnote"><Plus size={13}/> This device was not present in the previous scan.</p>}
                  {item.change === "removed" && <p className="trend-footnote"><Minus size={13}/> This device was not present in the latest scan.</p>}
                  {item.statusChanged && <p className="trend-status-change">Status changed: <strong>{item.previousStatus}</strong> → <strong>{item.currentStatus}</strong></p>}
                  {item.metrics.length > 0 ? (
                    <div className="trend-metrics">
                      {item.metrics.map((metric) => {
                        const positive = metric.delta > 0;
                        const isConcerning = isMetricWorsening(metric.metric, metric.delta);
                        const Icon = positive ? TrendingUp : TrendingDown;
                        return <div className="trend-metric" key={metric.metric}>
                          <span>{metric.label}</span>
                          <strong>{metric.previous}{metric.unit} → {metric.current}{metric.unit}</strong>
                          <small className={isConcerning ? "trend-value-warning" : "trend-value-neutral"}><Icon size={12}/>{formatMetricDelta(metric)}</small>
                        </div>;
                      })}
                    </div>
                  ) : item.change === "unchanged" ? <p className="trend-footnote">No comparable changes in the reported metrics.</p> : null}
                </article>
              ))}
            </div>
          )}
          <p className="trend-footnote">Trends describe changes in reported SMART evidence. They are not a prediction of failure or remaining service life. A change may reflect sensor variation, firmware reporting or device replacement.</p>
        </>
      )}
    </section>
  );
}
