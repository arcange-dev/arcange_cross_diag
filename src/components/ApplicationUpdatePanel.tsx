import { useState } from "react";
import { AlertTriangle, CheckCircle2, Download, LoaderCircle, RefreshCw, ShieldCheck } from "lucide-react";
import {
  checkForApplicationUpdate,
  installApplicationUpdate,
  isAppUpdaterEnabled,
  type AppUpdateMetadata,
  type AppUpdateProgress,
} from "../lib/appUpdater";

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index++;
  }
  return `${value.toFixed(1)} ${units[index]}`;
}

export function ApplicationUpdatePanel() {
  const enabled = isAppUpdaterEnabled();
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [update, setUpdate] = useState<AppUpdateMetadata | null>(null);
  const [checked, setChecked] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<AppUpdateProgress>({ downloadedBytes: 0, contentLength: null });

  const checkNow = async () => {
    setChecking(true);
    setError(null);
    setMessage(null);
    setUpdate(null);
    try {
      const result = await checkForApplicationUpdate();
      setUpdate(result);
      setChecked(true);
      setMessage(result ? `A signed update to version ${result.version} is available.` : "CrossDiag is up to date.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setChecking(false);
    }
  };

  const install = async () => {
    if (!update || installing) return;
    const accepted = window.confirm(
      `Install the signed CrossDiag update ${update.version} and restart the application? Save any open work first.`,
    );
    if (!accepted) return;

    setInstalling(true);
    setError(null);
    setMessage("Downloading and verifying the signed update…");
    try {
      await installApplicationUpdate(setProgress);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setMessage(null);
      setInstalling(false);
    }
  };

  const progressPercent = progress.contentLength && progress.contentLength > 0
    ? Math.min(100, Math.round((progress.downloadedBytes / progress.contentLength) * 100))
    : null;

  return (
    <section className="panel app-update-panel" aria-labelledby="app-update-title">
      <div className="app-update-heading">
        <div className="app-update-icon"><Download size={19} /></div>
        <div className="app-update-copy">
          <div className="eyebrow">SIGNED APPLICATION UPDATES</div>
          <h3 id="app-update-title">CrossDiag application updates</h3>
          <p>Check for signed releases. Installation is always started by you and requires confirmation.</p>
        </div>
        <button className="subtle-button" onClick={() => void checkNow()} disabled={!enabled || checking || installing}>
          {checking ? <LoaderCircle className="spin" size={14} /> : <RefreshCw size={14} />}
          {checking ? "Checking…" : "Check now"}
        </button>
      </div>

      {!enabled && (
        <div className="app-update-state state-muted">
          <ShieldCheck size={16} />
          <span>Updater is disabled in development/browser builds. Signed release builds enable it after the maintainer configures the updater public key and signing secret.</span>
        </div>
      )}
      {message && (
        <div className="app-update-state state-success">
          <CheckCircle2 size={16} />
          <span>{message}</span>
        </div>
      )}
      {error && (
        <div className="app-update-state state-error">
          <AlertTriangle size={16} />
          <span>{error}</span>
        </div>
      )}
      {update && (
        <div className="app-update-details">
          <div className="app-update-version-row">
            <div><span>Current version</span><strong>{update.currentVersion}</strong></div>
            <div><span>Available version</span><strong>{update.version}</strong></div>
            <button className="primary-button" onClick={() => void install()} disabled={installing}>
              {installing ? <LoaderCircle className="spin" size={14} /> : <Download size={14} />}
              {installing ? "Installing…" : "Download & install"}
            </button>
          </div>
          {update.date && <div className="app-update-date">Published: {update.date}</div>}
          <pre className="app-update-notes">{update.notes}</pre>
        </div>
      )}
      {installing && (
        <div className="app-update-progress">
          <div className="progress-track"><span style={{ width: `${progressPercent ?? 12}%` }} /></div>
          <span>{progressPercent === null ? `${formatBytes(progress.downloadedBytes)} downloaded` : `${progressPercent}% · ${formatBytes(progress.downloadedBytes)} / ${formatBytes(progress.contentLength ?? 0)}`}</span>
        </div>
      )}
      {checked && !update && !error && <div className="app-update-footnote">No new signed release was found. Check again after a new stable release is published.</div>}
    </section>
  );
}
