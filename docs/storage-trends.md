# Storage health trends and CSV export — v0.9.0

The Storage Diagnostics screen compares the two newest saved SMART scans and exports saved storage history as CSV.

## Comparison rules

- Scans are ordered by their collection timestamp, not by array order.
- Physical devices are matched by the local device path reported by the scan (`/dev/nvme0n1`, `PhysicalDrive0`, etc.). Device paths can change between boots or hardware reconfiguration; an unmatched path is shown as added/removed, not asserted to be a different physical drive.
- Only metrics present as finite numeric values in both scans are compared. A missing value is never converted to zero.
- The panel reports status changes, newly observed device paths, removed device paths and numeric differences.
- Power-on hours are cumulative usage and are not flagged as worsening merely because the number increased.
- Decreasing remaining endurance or spare capacity, and increasing endurance used, temperature, media errors, reallocated sectors, pending sectors or uncorrectable sectors are highlighted as potentially concerning trends. This is a triage cue, not a prediction of imminent failure.
- A single scan cannot establish a trend. At least two timestamped scans are required.

## CSV export

The Export CSV action downloads all locally saved storage scan rows, including scan time, configured threshold, drive path/model, health status, supported SMART metrics, findings and limitations. Missing fields remain empty. Serial numbers are not collected or exported by this feature.

The CSV is generated locally in the desktop webview. It is not uploaded. Review it before sharing because device paths, model names and diagnostic findings may still reveal information about your computer.

## Validation

Run:

```bash
npm run test:storage-trends
```

The tests cover chronological ordering, metric deltas, added/removed drive paths, missing data semantics, CSV quoting and direction-aware warning behavior.
