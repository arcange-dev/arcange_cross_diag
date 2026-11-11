import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import { ApplicationUpdatePanel } from "./components/ApplicationUpdatePanel";
import { StorageTrendPanel } from "./components/StorageTrendPanel";
import {
  Activity,
  BatteryCharging,
  CalendarClock,
  CirclePlay,
  HardDriveDownload,
  Printer,
  Save,
  Trash2,
  AlertTriangle,
  ArrowDownToLine,
  ArrowRight,
  BadgeCheck,
  BookOpenText,
  CheckCircle2,
  ChevronDown,
  CircleHelp,
  ClipboardList,
  Cpu,
  Download,
  ExternalLink,
  FileText,
  Gauge,
  HardDrive,
  Laptop,
  LayoutDashboard,
  LoaderCircle,
  MemoryStick,
  MonitorCog,
  Network,
  RefreshCw,
  Search,
  Settings2,
  ShieldCheck,
  Thermometer,
  Waves,
  Wrench,
  XCircle,
  type LucideIcon,
} from "lucide-react";
import {
  getSystemSnapshot,
  getRuntimeReadiness,
  getStorageHistory,
  scanStorageHealth,
  getHardwareDiagnostics,
  inspectRecoveryImage,
  listRecoveryMedia,
  checkUpdateStatus,
  analyzeSystemFaults,
  installSelectedUpdate,
  installOperatingSystemUpdates,
  getRepairCatalog,
  runRepairAction,
  listTechnicianCases,
  createTechnicianCase,
  updateTechnicianCase,
  deleteTechnicianCase,
  getScanSchedule,
  saveScanSchedule,
  openDriverManager,
  openSystemUpdateSettings,
  runPerformanceTest as executePerformanceTest,
  type PerformanceTestResult,
  type UpdateInstallResult,
  type RepairAction,
  type RepairResult,
  type TechnicianCase,
  type ScanSchedule,
  type StorageScan,
  type SystemSnapshot,
  type RuntimeReadiness,
  type HardwareDiagnostics,
  type UpdateStatus,
  type FaultReport,
  type RecoveryImageInfo,
  type RecoveryMediaTarget,
} from "./lib/desktop";

type Section = "overview" | "hardware" | "performance" | "storage" | "drivers" | "updates" | "repair" | "recovery" | "errors" | "cases" | "reports" | "settings";
type NavItem = { id: Section; label: string; icon: LucideIcon; group: "main" | "tools" | "system" };

const navigation: NavItem[] = [
  { id: "overview", label: "Overview", icon: LayoutDashboard, group: "main" },
  { id: "hardware", label: "Hardware inventory", icon: Cpu, group: "main" },
  { id: "performance", label: "Performance tests", icon: Gauge, group: "main" },
  { id: "storage", label: "Storage health", icon: HardDrive, group: "main" },
  { id: "drivers", label: "Drivers & devices", icon: Cpu, group: "main" },
  { id: "updates", label: "Update center", icon: ArrowDownToLine, group: "tools" },
  { id: "errors", label: "Fault analyzer", icon: Wrench, group: "tools" },
  { id: "repair", label: "Repair center", icon: Wrench, group: "tools" },
  { id: "recovery", label: "BIOS & recovery", icon: HardDriveDownload, group: "tools" },
  { id: "cases", label: "Technician cases", icon: ClipboardList, group: "tools" },
  { id: "reports", label: "Reports & history", icon: ClipboardList, group: "tools" },
  { id: "settings", label: "Settings", icon: Settings2, group: "system" },
];

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "Not reported";
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toFixed(value >= 100 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function percent(used: number, total: number): number {
  if (total <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((used / total) * 100)));
}

function dateLabel(value?: string): string {
  if (!value) return "Not collected yet";
  const normalized = value.startsWith("unix:") ? Number(value.slice(5)) * 1000 : value;
  const date = new Date(normalized);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export default function App() {
  const [active, setActive] = useState<Section>("overview");
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [runtimeReadiness, setRuntimeReadiness] = useState<RuntimeReadiness | null>(null);
  const [readinessLoading, setReadinessLoading] = useState(false);
  const [readinessAttempted, setReadinessAttempted] = useState(false);
  const [readinessError, setReadinessError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [storageScan, setStorageScan] = useState<StorageScan | null>(null);
  const [storageHistory, setStorageHistory] = useState<StorageScan[]>([]);
  const [hardwareDiagnostics, setHardwareDiagnostics] = useState<HardwareDiagnostics | null>(null);
  const [hardwareLoading, setHardwareLoading] = useState(false);
  const [hardwareError, setHardwareError] = useState<string | null>(null);
  const [performanceResult, setPerformanceResult] = useState<PerformanceTestResult | null>(null);
  const [performanceLoading, setPerformanceLoading] = useState<string | null>(null);
  const [performanceError, setPerformanceError] = useState<string | null>(null);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [updateLoading, setUpdateLoading] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [faultReport, setFaultReport] = useState<FaultReport | null>(null);
  const [faultLoading, setFaultLoading] = useState(false);
  const [faultError, setFaultError] = useState<string | null>(null);
  const [storageLoading, setStorageLoading] = useState(false);
  const [storageError, setStorageError] = useState<string | null>(null);
  const [installingPackage, setInstallingPackage] = useState<string | null>(null);
  const [installResult, setInstallResult] = useState<UpdateInstallResult | null>(null);
  const [repairCatalog, setRepairCatalog] = useState<RepairAction[]>([]);
  const [repairResult, setRepairResult] = useState<RepairResult | null>(null);
  const [repairRunning, setRepairRunning] = useState<string | null>(null);
  const [repairError, setRepairError] = useState<string | null>(null);
  const [cases, setCases] = useState<TechnicianCase[]>([]);
  const [schedule, setSchedule] = useState<ScanSchedule>({ enabled: false, interval_hours: 24, threshold_percent: 90 });
  const [scheduleReady, setScheduleReady] = useState(false);
  const [caseError, setCaseError] = useState<string | null>(null);
  const [driverNotice, setDriverNotice] = useState<string | null>(null);
  const [thresholdPercent, setThresholdPercent] = useState(() => {
    try {
      const saved = Number(window.localStorage.getItem("crossdiag-storage-threshold"));
      return Number.isFinite(saved) && saved >= 0 && saved <= 100 ? saved : 90;
    } catch {
      return 90;
    }
  });

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    setNotice(null);
    try {
      const data = await getSystemSnapshot();
      setSnapshot(data);
      setNotice("System inventory refreshed. No repair or update actions were performed.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const refreshRuntimeReadiness = useCallback(async () => {
    setReadinessAttempted(true);
    setReadinessLoading(true);
    setReadinessError(null);
    try {
      setRuntimeReadiness(await getRuntimeReadiness());
    } catch (e) {
      setReadinessError(e instanceof Error ? e.message : String(e));
    } finally {
      setReadinessLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    void getStorageHistory().then(setStorageHistory).catch(() => undefined);
    void listTechnicianCases().then(setCases).catch((e) => setCaseError(e instanceof Error ? e.message : String(e)));
    void getScanSchedule().then((value) => { setSchedule(value); setThresholdPercent(value.threshold_percent); setScheduleReady(true); }).catch(() => setScheduleReady(true));
    void getRepairCatalog().then(setRepairCatalog).catch(() => undefined);
  }, [refresh]);

  useEffect(() => {
    if (active === "settings" && !readinessAttempted) {
      void refreshRuntimeReadiness();
    }
  }, [active, readinessAttempted, refreshRuntimeReadiness]);

  useEffect(() => {
    try {
      window.localStorage.setItem("crossdiag-storage-threshold", String(thresholdPercent));
    } catch {
      // Preference persistence is optional; the current session still uses the selected value.
    }
  }, [thresholdPercent]);

  useEffect(() => {
    if (!scheduleReady || schedule.threshold_percent === thresholdPercent) return;
    void saveScanSchedule(schedule.enabled, schedule.interval_hours, thresholdPercent)
      .then(setSchedule)
      .catch((e) => setCaseError(e instanceof Error ? e.message : String(e)));
  }, [scheduleReady, schedule.enabled, schedule.interval_hours, schedule.threshold_percent, thresholdPercent]);

  useEffect(() => {
    if (!scheduleReady || !schedule.enabled) return;
    // The native Rust scheduler performs scans while the desktop process is running.
    // This frontend timer only refreshes the displayed local history; it does not start scans.
    const timer = window.setInterval(async () => {
      try {
        const history = await getStorageHistory();
        setStorageHistory(history);
        if (history.length) setStorageScan(history[history.length - 1]);
      } catch (e) {
        setStorageError(e instanceof Error ? e.message : String(e));
      }
    }, 30_000);
    return () => window.clearInterval(timer);
  }, [scheduleReady, schedule.enabled]);

  const runHardwareDiagnostics = useCallback(async () => {
    setHardwareLoading(true);
    setHardwareError(null);
    try {
      const result = await getHardwareDiagnostics();
      setHardwareDiagnostics(result);
      setNotice("Hardware evidence collected. No stress tests, repairs or firmware changes were run.");
    } catch (e) {
      setHardwareError(e instanceof Error ? e.message : String(e));
    } finally {
      setHardwareLoading(false);
    }
  }, []);

  const runPerformanceCheck = useCallback(async (testId: "cpu_single_core" | "cpu_multi_core" | "memory_smoke", memoryMb = 64) => {
    const details = testId === "cpu_single_core"
      ? "Run a single-core calculation benchmark for five seconds. It may increase CPU use on one logical core. Save work first and stop other benchmarks."
      : testId === "cpu_multi_core"
        ? "Run a bounded multi-core calculation benchmark for five seconds using up to eight worker threads. CPU temperature and fan telemetry may not be available on every PC. Save work first and stop other benchmarks."
        : `Allocate and verify ${memoryMb} MiB of application memory using six patterns. This is a bounded user-space smoke test, not a full physical RAM stability test. Close memory-heavy apps first.`;
    const testLabel = testId === "cpu_single_core" ? "Single-core CPU benchmark" : testId === "cpu_multi_core" ? "Multi-core CPU benchmark" : "RAM smoke test";
    if (!window.confirm(`${testLabel}

${details}

Continue?`)) return;
    setPerformanceLoading(testId);
    setPerformanceError(null);
    setPerformanceResult(null);
    try {
      setPerformanceResult(await executePerformanceTest(testId, 5, memoryMb));
    } catch (e) {
      setPerformanceError(e instanceof Error ? e.message : String(e));
    } finally {
      setPerformanceLoading(null);
    }
  }, []);

  const runUpdateCheck = useCallback(async () => {
    setUpdateLoading(true);
    setUpdateError(null);
    try {
      setUpdateStatus(await checkUpdateStatus());
    } catch (e) {
      setUpdateError(e instanceof Error ? e.message : String(e));
    } finally {
      setUpdateLoading(false);
    }
  }, []);

  const installUpdate = useCallback(async (update: UpdateStatus["updates"][number]) => {
    if (!update.install_id) return;
    const confirmed = window.confirm(`Install update?\n\n${update.name}\nSource: ${update.source}\nVersion: ${update.version ?? "Not reported"}\n\nThis changes installed software. Back up important data and save open work first.`);
    if (!confirmed) return;
    setInstallingPackage(update.install_id);
    setInstallResult(null);
    setUpdateError(null);
    try {
      const result = await installSelectedUpdate(update.install_id, update.source);
      setInstallResult(result);
      if (result.success) {
        setNotice(`Update command completed for ${update.name}. Run another update check to verify the installed version.`);
        await runUpdateCheck();
      } else {
        setUpdateError(`Update command returned exit code ${result.exit_code ?? "unknown"}. Review the installation output below.`);
      }
    } catch (e) {
      setUpdateError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstallingPackage(null);
    }
  }, [runUpdateCheck]);

  const installOsUpdates = useCallback(async () => {
    const confirmed = window.confirm("Install distribution-wide Linux updates?\n\nThis may update kernel, system libraries and packages used by other applications. A restart may be needed. Save your work and back up important data first. Continue?");
    if (!confirmed) return;
    setInstallingPackage("__system_updates__");
    setInstallResult(null);
    setUpdateError(null);
    try {
      const result = await installOperatingSystemUpdates();
      setInstallResult(result);
      if (result.success) {
        setNotice("The package-manager upgrade command completed. Restart only if your distribution indicates it is required, then run another update check.");
        await runUpdateCheck();
      } else {
        setUpdateError(`The distribution update command returned exit code ${result.exit_code ?? "unknown"}. Review its output.`);
      }
    } catch (e) {
      setUpdateError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstallingPackage(null);
    }
  }, [runUpdateCheck]);

  const executeRepairAction = useCallback(async (action: RepairAction) => {
    const confirmed = window.confirm(`${action.title}\n\n${action.description}\n\nCrossDiag will run only this fixed, allow-listed action. Continue?`);
    if (!confirmed) return;
    setRepairRunning(action.id);
    setRepairError(null);
    setRepairResult(null);
    try {
      setRepairResult(await runRepairAction(action.id));
    } catch (e) {
      setRepairError(e instanceof Error ? e.message : String(e));
    } finally {
      setRepairRunning(null);
    }
  }, []);

  const refreshCases = useCallback(async () => {
    try { setCases(await listTechnicianCases()); setCaseError(null); }
    catch (e) { setCaseError(e instanceof Error ? e.message : String(e)); }
  }, []);

  const createCase = useCallback(async (title: string, device: string, notes: string): Promise<boolean> => {
    try { await createTechnicianCase(title, device, notes); await refreshCases(); return true; }
    catch (e) { setCaseError(e instanceof Error ? e.message : String(e)); return false; }
  }, [refreshCases]);

  const updateCase = useCallback(async (id: number, status: string, notes: string) => {
    try { await updateTechnicianCase(id, status, notes); await refreshCases(); }
    catch (e) { setCaseError(e instanceof Error ? e.message : String(e)); }
  }, [refreshCases]);

  const removeCase = useCallback(async (id: number) => {
    if (!window.confirm("Delete this local technician case permanently?")) return;
    try { await deleteTechnicianCase(id); await refreshCases(); }
    catch (e) { setCaseError(e instanceof Error ? e.message : String(e)); }
  }, [refreshCases]);

  const changeSchedule = useCallback(async (enabled: boolean, intervalHours: number) => {
    try { setSchedule(await saveScanSchedule(enabled, intervalHours, thresholdPercent)); setScheduleReady(true); }
    catch (e) { setCaseError(e instanceof Error ? e.message : String(e)); }
  }, [thresholdPercent]);

  const runDriverManager = useCallback(async () => {
    try { setDriverNotice(await openDriverManager()); }
    catch (e) { setDriverNotice(e instanceof Error ? e.message : String(e)); }
  }, []);

  const openOsUpdates = useCallback(async () => {
    try { setNotice(await openSystemUpdateSettings()); }
    catch (e) { setUpdateError(e instanceof Error ? e.message : String(e)); }
  }, []);

  const runFaultAnalysis = useCallback(async () => {
    setFaultLoading(true);
    setFaultError(null);
    try {
      setFaultReport(await analyzeSystemFaults());
    } catch (e) {
      setFaultError(e instanceof Error ? e.message : String(e));
    } finally {
      setFaultLoading(false);
    }
  }, []);

  const runStorageScan = useCallback(async () => {
    setStorageLoading(true);
    setStorageError(null);
    try {
      const result = await scanStorageHealth(thresholdPercent);
      setStorageScan(result);
      const history = await getStorageHistory();
      setStorageHistory(history);
      if (result.scan_status === "tool_missing") {
        setStorageError("smartmontools was not found. Install it using the instructions below, then scan again.");
      }
    } catch (e) {
      setStorageError(e instanceof Error ? e.message : String(e));
    } finally {
      setStorageLoading(false);
    }
  }, [thresholdPercent]);

  const visibleDrives = useMemo(() => {
    const drives = snapshot?.drives ?? [];
    const q = query.trim().toLowerCase();
    if (!q) return drives;
    return drives.filter((d) => `${d.name} ${d.mount_point} ${d.file_system}`.toLowerCase().includes(q));
  }, [snapshot, query]);

  const title = navigation.find((item) => item.id === active)?.label ?? "Overview";
  const usedMemory = snapshot ? percent(snapshot.used_memory_bytes, snapshot.total_memory_bytes) : 0;

  const navigate = (section: Section) => {
    setActive(section);
    setError(null);
    setNotice(null);
  };

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <div className="brand-mark"><MonitorCog size={23} strokeWidth={1.8} /></div>
          <div className="brand-copy"><strong>CrossDiag</strong><span>ARCANGE TECH LABS</span></div>
          <span className="version-chip">0.10.0</span>
        </div>
        <button className="device-selector" onClick={() => void refresh()} title="Refresh detected PC">
          <div className="device-icon"><Laptop size={18} /></div>
          <div className="device-copy"><strong>{snapshot?.hostname ?? "This computer"}</strong><span>{snapshot?.os_name ?? "Detecting operating system…"}</span></div>
          {loading ? <LoaderCircle className="spin" size={16} /> : <ChevronDown size={15} className="muted-icon" />}
        </button>
        <div className="nav-scroll">
          <NavGroup label="WORKSPACE" items={navigation.filter((n) => n.group === "main")} active={active} onNavigate={navigate} />
          <NavGroup label="DIAGNOSTICS" items={navigation.filter((n) => n.group === "tools")} active={active} onNavigate={navigate} />
          <NavGroup label="PREFERENCES" items={navigation.filter((n) => n.group === "system")} active={active} onNavigate={navigate} />
        </div>
        <div className="sidebar-bottom">
          <div className="opensource-card"><div className="opensource-icon"><ShieldCheck size={16} /></div><div><strong>Open-source by design</strong><span>Local-first. No cloud account.</span></div></div>
          <div className="sidebar-footer"><span className="status-dot" /> Local diagnostic mode <span className="footer-separator">·</span> <span>v0.10.0</span></div>
        </div>
      </aside>

      <main className="main-area">
        <header className="topbar">
          <div className="breadcrumbs"><span>CrossDiag</span><span className="crumb-slash">/</span><strong>{title}</strong></div>
          <div className="topbar-actions">
            <div className="local-badge"><span className="status-dot" /> Local only</div>
            <button className="icon-button" title="Refresh system inventory" onClick={() => void refresh()} disabled={loading}>{loading ? <LoaderCircle className="spin" size={17} /> : <RefreshCw size={17} />}</button>
            <button className="profile-button" title="Project information" onClick={() => navigate("settings")}>A</button>
          </div>
        </header>

        <div className="page-content">
          <div className="page-heading">
            <div><div className="eyebrow"><span className="eyebrow-line" /> COMPUTER SYSTEMS / OVERVIEW</div><h1>{pageTitle(active)}</h1><p>{pageDescription(active)}</p></div>
            <div className="heading-actions"><span className="last-scan">{snapshot ? `Updated ${dateLabel(snapshot.collected_at)}` : "Inventory not collected"}</span><button className="primary-button" onClick={() => void refresh()} disabled={loading}>{loading ? <LoaderCircle className="spin" size={16} /> : <Activity size={16} />}{loading ? "Scanning…" : "Refresh inventory"}</button></div>
          </div>

          {error && <div className="alert alert-error"><XCircle size={18} /><div><strong>Could not collect live system data</strong><span>{error}</span></div><button onClick={() => void refresh()} className="alert-action">Retry</button></div>}
          {notice && !error && <div className="alert alert-success"><CheckCircle2 size={17} /><span>{notice}</span><button className="alert-close" aria-label="Dismiss message" onClick={() => setNotice(null)}>×</button></div>}

          {active === "overview" && <Overview snapshot={snapshot} loading={loading} usedMemory={usedMemory} storageScan={storageScan} hardwareDiagnostics={hardwareDiagnostics} updateStatus={updateStatus} onNavigate={navigate} onRefresh={() => void refresh()} />}
          {active === "hardware" && <HardwarePage snapshot={snapshot} diagnostics={hardwareDiagnostics} loading={loading || hardwareLoading} error={hardwareError} usedMemory={usedMemory} onScan={() => void runHardwareDiagnostics()} />}
          {active === "performance" && <PerformancePage result={performanceResult} loading={performanceLoading} error={performanceError} onRun={(testId, memoryMb) => void runPerformanceCheck(testId, memoryMb)} />}
          {active === "drivers" && <DriversPage diagnostics={hardwareDiagnostics} snapshot={snapshot} loading={hardwareLoading} notice={driverNotice} onScan={() => void runHardwareDiagnostics()} onOpenManager={() => void runDriverManager()} />}
          {active === "storage" && <StoragePage
            drives={visibleDrives}
            query={query}
            setQuery={setQuery}
            snapshot={snapshot}
            storageScan={storageScan}
            storageHistory={storageHistory}
            storageLoading={storageLoading}
            storageError={storageError}
            thresholdPercent={thresholdPercent}
            setThresholdPercent={setThresholdPercent}
            onScan={() => void runStorageScan()}
          />}
          {active === "updates" && <UpdateCenterPage status={updateStatus} loading={updateLoading} error={updateError} installing={installingPackage} installResult={installResult} onCheck={() => void runUpdateCheck()} onOpenOsUpdates={() => void openOsUpdates()} onInstallOsUpdates={() => void installOsUpdates()} onInstall={(update) => void installUpdate(update)} />}
          {active === "repair" && <RepairCenterPage catalog={repairCatalog} result={repairResult} running={repairRunning} error={repairError} onRun={(action) => void executeRepairAction(action)} />}
          {active === "recovery" && <RecoveryPage manufacturer={snapshot?.manufacturer ?? ""} />}
          {active === "cases" && <CasesPage cases={cases} error={caseError} schedule={schedule} onCreate={(title, device, notes) => createCase(title, device, notes)} onUpdate={(id, status, notes) => void updateCase(id, status, notes)} onDelete={(id) => void removeCase(id)} onSchedule={(enabled, hours) => void changeSchedule(enabled, hours)} />}
          {active === "errors" && <FaultAnalyzerPage report={faultReport} loading={faultLoading} error={faultError} onScan={() => void runFaultAnalysis()} />}
          {active === "reports" && <ReportsPage snapshot={snapshot} storageScan={storageScan} storageHistory={storageHistory} hardwareDiagnostics={hardwareDiagnostics} updateStatus={updateStatus} faultReport={faultReport} performanceResult={performanceResult} cases={cases} />}
          {active === "settings" && <SettingsPage snapshot={snapshot} readiness={runtimeReadiness} loading={readinessLoading} error={readinessError} onCheck={() => void refreshRuntimeReadiness()} />}

          <footer className="page-footer"><span>ARCANGE CROSSDIAG <span className="footer-dot">•</span> OPEN-SOURCE PC DIAGNOSTICS</span><span>Built for clarity. Designed for safe repair.</span></footer>
        </div>
      </main>
    </div>
  );
}

function NavGroup({ label, items, active, onNavigate }: { label: string; items: NavItem[]; active: Section; onNavigate: (s: Section) => void }) {
  return <div className="nav-group"><div className="nav-label">{label}</div>{items.map((item) => { const Icon = item.icon; return <button key={item.id} className={`nav-item ${active === item.id ? "active" : ""}`} onClick={() => onNavigate(item.id)}><Icon size={17} strokeWidth={1.9} /><span>{item.label}</span>{item.id === "storage" && <span className="nav-next"><ArrowRight size={13} /></span>}</button>; })}</div>;
}

function pageTitle(section: Section): string {
  const titles: Record<Section, string> = { overview: "System overview", hardware: "Hardware inventory", performance: "Performance tests", storage: "Storage diagnostics", drivers: "Drivers & devices", updates: "Update center", repair: "Repair center", recovery: "BIOS & recovery", errors: "Fault analyzer", cases: "Technician cases", reports: "Reports & history", settings: "Settings & about" };
  return titles[section];
}
function pageDescription(section: Section): string {
  const descriptions: Record<Section, string> = {
    overview: "Your PC at a glance. Gather real system data before making a diagnosis.",
    hardware: "Identify the components installed in this computer and the information the OS can report.",
    performance: "Run short, bounded CPU and memory smoke tests; these do not replace full hardware stress diagnostics.",
    storage: "Inspect physical drives with SMART evidence, endurance estimates and mounted-volume capacity.",
    drivers: "Review installed drivers and problem devices, then continue with operating-system or vendor tools.",
    updates: "Discover pending operating-system and supported package updates without installing them.",
    errors: "Review recent system error events and understand their diagnostic limitations.",
    repair: "Run narrowly scoped, allow-listed diagnostics and safe maintenance actions after confirmation.",
    recovery: "Prepare for firmware setup, operating-system recovery and bootable installation media.",
    cases: "Keep local repair cases and opt into periodic storage checks while CrossDiag is open.",
    reports: "Local-first session history and structured diagnostic reports.",
    settings: "Application information, privacy, update settings and runtime tool readiness.",
  };
  return descriptions[section];
}

function Overview({ snapshot, loading, usedMemory, storageScan, hardwareDiagnostics, updateStatus, onNavigate, onRefresh }: { snapshot: SystemSnapshot | null; loading: boolean; usedMemory: number; storageScan: StorageScan | null; hardwareDiagnostics: HardwareDiagnostics | null; updateStatus: UpdateStatus | null; onNavigate: (s: Section) => void; onRefresh: () => void }) {
  return <>
    <section className="hero-card">
      <div className="hero-art"><div className="hero-orbit orbit-one"/><div className="hero-orbit orbit-two"/><div className="hero-chip"><MonitorCog size={38} strokeWidth={1.4}/><span className="hero-chip-dot"/></div><div className="hero-spark spark-one"/><div className="hero-spark spark-two"/><div className="hero-spark spark-three"/></div>
      <div className="hero-copy"><div className="hero-pill"><span className="status-dot"/> CROSS-PLATFORM TECHNICIAN SUITE</div><h2>Know your system.<br/><span>Fix it with confidence.</span></h2><p>Collect system facts, examine hardware evidence, understand errors and follow safe troubleshooting steps — all from one local application.</p><div className="hero-actions"><button className="hero-button" onClick={onRefresh} disabled={loading}>{loading ? <LoaderCircle className="spin" size={16}/> : <Activity size={16}/>} {loading ? "Collecting…" : "Collect system facts"}<ArrowRight size={15}/></button><button className="text-button" onClick={() => onNavigate("hardware")}>Explore hardware <ArrowRight size={14}/></button></div></div>
      <div className="hero-bottom-line"><span><ShieldCheck size={14}/> Local-first diagnostics</span><span><BadgeCheck size={14}/> Evidence before recommendations</span><span><Waves size={14}/> Windows + Linux roadmap</span></div>
    </section>

    <div className="section-line"><div><h2>System snapshot</h2><p>Live values reported by the current operating system</p></div><span className={`snapshot-status ${snapshot ? "ready" : "pending"}`}><span className="status-dot"/>{snapshot ? "Inventory available" : "Waiting for desktop runtime"}</span></div>
    <section className="metric-grid">
      <MetricCard icon={Cpu} label="Processor" value={snapshot?.cpu_name ?? "Not collected"} foot={snapshot ? `${snapshot.logical_cores} logical threads${snapshot.physical_cores ? ` · ${snapshot.physical_cores} physical cores` : ""}` : "Live data requires desktop app"} accent="cyan" />
      <MetricCard icon={MemoryStick} label="System memory" value={snapshot ? formatBytes(snapshot.total_memory_bytes) : "Not collected"} foot={snapshot ? `${formatBytes(snapshot.used_memory_bytes)} used · ${usedMemory}% of detected RAM` : "Capacity and usage will appear here"} accent="violet" progress={snapshot ? usedMemory : undefined} />
      <MetricCard icon={HardDrive} label="Detected volumes" value={snapshot ? String(snapshot.drives.length) : "—"} foot={snapshot ? "Mounted disks and volumes detected" : "Disk inventory not collected"} accent="blue" />
      <MetricCard icon={Gauge} label="Health verdict" value="Not assessed" foot="No component lifetime or failure score is fabricated" accent="amber" status="neutral" />
    </section>

    <div className="content-grid">
      <section className="panel hardware-panel"><div className="panel-header"><div><h3>Hardware overview</h3><p>Inventory collection status by subsystem</p></div><button className="subtle-button" onClick={() => onNavigate("hardware")}>View inventory <ArrowRight size={14}/></button></div>
        <div className="subsystem-list"><Subsystem icon={Cpu} title="CPU & motherboard" detail={snapshot?.cpu_name ?? "Awaiting system inventory"} status={snapshot ? "Collected" : "Not collected"} tone={snapshot ? "info" : "idle"}/><Subsystem icon={MemoryStick} title="Memory (RAM)" detail={snapshot ? `${formatBytes(snapshot.total_memory_bytes)} detected` : "Capacity and usage"} status={snapshot ? "Collected" : "Not collected"} tone={snapshot ? "info" : "idle"}/><Subsystem icon={HardDrive} title="Storage devices" detail={storageScan ? `${storageScan.disks.length} physical drive(s) checked` : snapshot ? `${snapshot.drives.length} mounted volume(s) · SMART scan available` : "Disk identification & SMART"} status={storageScan ? storageScan.scan_status.replaceAll("_", " ") : snapshot ? "Inventory collected" : "Not collected"} tone={storageScan?.disks.some((disk) => disk.status === "failed") ? "warning" : storageScan ? "info" : snapshot ? "warning" : "idle"}/><Subsystem icon={Thermometer} title="Temperatures & cooling" detail={hardwareDiagnostics ? `${hardwareDiagnostics.sensors.length} sensor reading(s) reported` : "Run a hardware evidence scan"} status={hardwareDiagnostics ? "Collected" : "Not scanned"} tone={hardwareDiagnostics ? "info" : "idle"}/><Subsystem icon={Network} title="Network & peripherals" detail={hardwareDiagnostics ? `${hardwareDiagnostics.network_adapters.length} adapter(s) enumerated` : "Adapter inventory available in hardware scan"} status={hardwareDiagnostics ? "Collected" : "Not scanned"} tone={hardwareDiagnostics ? "info" : "idle"}/></div>
      </section>
      <section className="panel actions-panel"><div className="panel-header"><div><h3>Technician actions</h3><p>Start with safe, read-only checks</p></div><div className="panel-icon"><Wrench size={17}/></div></div><ActionRow icon={Laptop} title="System inventory" detail="OS, CPU, RAM and disks" onClick={() => onNavigate("hardware")}/><ActionRow icon={HardDrive} title="Review storage" detail="See detected volumes" onClick={() => onNavigate("storage")}/><ActionRow icon={AlertTriangle} title="Understand diagnostics" detail="See the fault-analysis roadmap" onClick={() => onNavigate("errors")}/><ActionRow icon={FileText} title="Reports & history" detail="Review report capabilities" onClick={() => onNavigate("reports")}/><div className="safety-note"><ShieldCheck size={15}/><span>Read-only foundation. No repairs, updates or firmware changes run automatically.</span></div></section>
    </div>

    <section className="panel readiness-panel"><div className="readiness-head"><div className="readiness-icon"><BookOpenText size={19}/></div><div><h3>Diagnostic coverage, built honestly</h3><p>CrossDiag never labels missing sensor data as a pass or invents a remaining-life percentage.</p></div><button className="subtle-button" onClick={() => onNavigate("settings")}>Project details <ArrowRight size={14}/></button></div><div className="coverage-grid"><Coverage icon={CheckCircle2} title="Live system facts" value={snapshot ? "Available" : "Waiting"} tone={snapshot ? "success" : "muted"}/><Coverage icon={HardDrive} title="SMART drive health" value={storageScan ? `${storageScan.disks.length} drive(s) scanned` : "Ready to scan"} tone={storageScan ? "success" : "muted"}/><Coverage icon={Thermometer} title="Sensor inventory" value={hardwareDiagnostics ? `${hardwareDiagnostics.sensors.length} reading(s)` : "Run hardware scan"} tone={hardwareDiagnostics ? "success" : "muted"}/><Coverage icon={ArrowDownToLine} title="OS update discovery" value={updateStatus ? updateStatus.status.replaceAll("_", " ") : "Not checked"} tone={updateStatus ? "success" : "muted"}/></div></section>
  </>;
}

function MetricCard({ icon: Icon, label, value, foot, accent, progress, status }: { icon: LucideIcon; label: string; value: string; foot: string; accent: string; progress?: number; status?: "neutral" }) {
  return <div className={`metric-card accent-${accent}`}><div className="metric-top"><span>{label}</span><div className="metric-icon"><Icon size={18}/></div></div><div className={`metric-value ${value.length > 22 ? "small-value" : ""}`}>{value}</div>{progress !== undefined && <div className="progress-track"><div className="progress-fill" style={{ width: `${progress}%` }}/></div>}<div className="metric-foot">{status && <CircleHelp size={13} />}{foot}</div></div>;
}
function Subsystem({ icon: Icon, title, detail, status, tone }: { icon: LucideIcon; title: string; detail: string; status: string; tone: string }) {
  return <div className="subsystem-row"><div className="subsystem-icon"><Icon size={17}/></div><div className="subsystem-copy"><strong>{title}</strong><span>{detail}</span></div><span className={`state-label ${tone}`}><span/>{status}</span></div>;
}
function ActionRow({ icon: Icon, title, detail, onClick }: { icon: LucideIcon; title: string; detail: string; onClick: () => void }) {
  return <button className="action-row" onClick={onClick}><div className="action-icon"><Icon size={17}/></div><div className="action-copy"><strong>{title}</strong><span>{detail}</span></div><ArrowRight size={15} className="action-arrow"/></button>;
}
function Coverage({ icon: Icon, title, value, tone }: { icon: LucideIcon; title: string; value: string; tone: string }) {
  return <div className="coverage-item"><div className={`coverage-icon ${tone}`}><Icon size={16}/></div><div><strong>{title}</strong><span>{value}</span></div></div>;
}

function biosEntryHint(manufacturer: string | undefined, mode: "setup" | "boot"): string {
  const brand = (manufacturer ?? "").toLowerCase();
  const hints: [string, string, string][] = [
    ["hp|hewlett", "Esc, then F10", "Esc, then F9"],
    ["dell", "F2", "F12"],
    ["lenovo", "F1 or F2 (some models use the Novo button)", "F12"],
    ["acer", "F2", "F12"],
    ["asus", "F2 or Delete", "F8"],
    ["msi|micro-star", "Delete", "F11"],
    ["gigabyte", "Delete", "F12"],
    ["surface|microsoft", "Hold Volume Up while pressing Power", "Use UEFI firmware boot options or the Surface boot sequence"],
  ];
  const match = hints.find(([pattern]) => new RegExp(pattern).test(brand));
  if (!match) return "Often F2, Delete, Esc, F10 or F12; confirm in the exact model's official manual.";
  return `${mode === "setup" ? match[1] : match[2]} — check the exact model manual.`;
}

function HardwarePage({ snapshot, diagnostics, loading, error, usedMemory, onScan }: { snapshot: SystemSnapshot | null; diagnostics: HardwareDiagnostics | null; loading: boolean; error: string | null; usedMemory: number; onScan: () => void }) {
  if (loading && !snapshot) return <LoadingState label="Collecting system inventory…"/>;
  const formatDate = (value?: string) => {
    if (!value) return "Not collected";
    const raw = value.startsWith("unix:") ? Number(value.slice(5)) * 1000 : value;
    const date = new Date(raw);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
  };
  const gpuRows: [string, string | undefined][] = [];
  diagnostics?.gpus.forEach((gpu, i) => gpuRows.push([`GPU ${i + 1}`, gpu.name], ["Driver", gpu.driver ?? "Not reported"], ["VRAM", gpu.vram_bytes ? formatBytes(gpu.vram_bytes) : "Not reported"], ["Status", gpu.status]));
  if (!gpuRows.length) gpuRows.push(["Detected", "No GPU information returned"]);
  const batteryRows: [string, string | undefined][] = [];
  diagnostics?.batteries.forEach((battery, i) => batteryRows.push([`Battery ${i + 1}`, battery.name], ["Charge", battery.charge_percent === null ? "Not reported" : `${battery.charge_percent}%`], ["Capacity retention estimate", battery.health_percent === null ? "Unavailable" : `${battery.health_percent.toFixed(1)}%`], ["Design capacity", battery.design_capacity_mwh === null ? "Not reported" : `${battery.design_capacity_mwh.toLocaleString()} mWh`], ["Full-charge capacity", battery.full_charge_capacity_mwh === null ? "Not reported" : `${battery.full_charge_capacity_mwh.toLocaleString()} mWh`], ["Status", battery.status ?? "Not reported"]));
  if (!batteryRows.length) batteryRows.push(["Detected", "No battery reported"]);
  const memoryRows: [string, string | undefined][] = [];
  diagnostics?.memory_modules.forEach((module, i) => memoryRows.push([`Module ${i + 1}`, module.locator ?? "Slot not reported"], ["Manufacturer", module.manufacturer ?? "Not reported"], ["Part number", module.part_number ?? "Not reported"], ["Capacity", module.capacity_bytes ? formatBytes(module.capacity_bytes) : "Not reported"], ["Speed", module.configured_speed_mhz ? `${module.configured_speed_mhz} MHz configured` : module.speed_mhz ? `${module.speed_mhz} MHz` : "Not reported"]));
  if (!memoryRows.length) memoryRows.push(["Module detail", "Not exposed by this OS or hardware"]);
  const firmwareRows: [string, string | undefined][] = diagnostics ? [["Vendor", diagnostics.firmware.vendor ?? "Not reported"], ["Version", diagnostics.firmware.version ?? "Not reported"], ["Release date", diagnostics.firmware.release_date ?? "Not reported"], ["Boot mode", diagnostics.firmware.mode ?? "Not reported"], ["Secure Boot", diagnostics.firmware.secure_boot === null ? "Not exposed" : diagnostics.firmware.secure_boot ? "Enabled" : "Disabled"], ["TPM", diagnostics.firmware.tpm_detected === null ? "Not exposed" : diagnostics.firmware.tpm_detected ? "Detected" : "Not detected"]] : [];
  const networkRows: [string, string | undefined][] = [];
  diagnostics?.network_adapters.forEach((adapter, i) => networkRows.push([`Adapter ${i + 1}`, adapter.name], ["State", adapter.status ?? "Not reported"], ["Link speed", adapter.link_speed ?? "Not reported"], ["Received (cumulative)", formatBytes(adapter.received_bytes)], ["Transmitted (cumulative)", formatBytes(adapter.transmitted_bytes)], ["MAC", adapter.mac_address ?? "Not reported"]));
  if (!networkRows.length) networkRows.push(["Detected", "No adapter information returned"]);
  return <div className="stack-layout">
    <div className="inventory-summary"><div><span className="eyebrow">DETECTED ENVIRONMENT</span><h2>{snapshot?.hostname ?? "No live snapshot"}</h2><p>{snapshot ? `${snapshot.manufacturer} ${snapshot.model} · ${snapshot.os_name} ${snapshot.os_version} · ${snapshot.architecture}` : "Run the desktop application to collect real system information."}</p></div><div className="inventory-badge"><Laptop size={18}/><span>{snapshot ? "System detected" : "Awaiting data"}</span></div><button className="primary-button" onClick={onScan} disabled={loading}>{loading ? <LoaderCircle className="spin" size={15}/> : <Activity size={15}/>} {loading ? "Scanning hardware…" : "Run hardware scan"}</button></div>
    {error && <div className="alert alert-error"><XCircle size={17}/><div><strong>Hardware scan failed</strong><span>{error}</span></div></div>}
    <div className="detail-grid"><DetailCard icon={Laptop} title="Operating system" rows={[["Name", snapshot?.os_name], ["Version", snapshot?.os_version], ["Kernel", snapshot?.kernel_version], ["Architecture", snapshot?.architecture], ["Hostname", snapshot?.hostname]]}/><DetailCard icon={Cpu} title="Processor" rows={[["CPU", snapshot?.cpu_name], ["Logical threads", snapshot?.logical_cores.toString()], ["Physical cores", snapshot?.physical_cores?.toString()], ["Current reported load", diagnostics ? `${diagnostics.cpu_usage_percent.toFixed(1)}%` : undefined], ["Reported frequency", diagnostics?.cpu_frequency_mhz ? `${diagnostics.cpu_frequency_mhz} MHz` : undefined]]}/><DetailCard icon={MemoryStick} title="Memory" rows={[["Installed / visible", snapshot ? formatBytes(snapshot.total_memory_bytes) : undefined], ["Currently used", snapshot ? formatBytes(snapshot.used_memory_bytes) : undefined], ["Usage", snapshot ? `${usedMemory}%` : undefined], ["Available", diagnostics ? formatBytes(diagnostics.available_memory_bytes) : undefined]]}/><DetailCard icon={MemoryStick} title="Memory modules" rows={memoryRows}/><DetailCard icon={HardDrive} title="Storage inventory" rows={[["Mounted volumes", snapshot?.drives.length.toString()], ["Inventory timestamp", dateLabel(snapshot?.collected_at)]]}/></div>
    {diagnostics && <>
      <div className="section-line"><div><h2>Hardware evidence</h2><p>Collected {formatDate(diagnostics.scanned_at)} · no stress tests were run</p></div><span className="tiny-tag">{diagnostics.gpus.length} GPU(s) · {diagnostics.sensors.length} sensor(s)</span></div>
      <div className="detail-grid"><DetailCard icon={MonitorCog} title="Graphics adapters" rows={gpuRows}/><DetailCard icon={BatteryCharging} title="Battery" rows={batteryRows}/><DetailCard icon={MonitorCog} title="BIOS / UEFI" rows={firmwareRows}/><DetailCard icon={Network} title="Network adapters" rows={networkRows}/></div>
      <div className="panel"><div className="panel-header"><div><h3>Per-core CPU activity</h3><p>Short system sample from the operating-system CPU counters. It is not a processor stress test.</p></div><span className="tiny-tag">{diagnostics.cpu_cores.length} logical processor(s)</span></div>{diagnostics.cpu_cores.length ? <div className="core-load-grid">{diagnostics.cpu_cores.map((core, index) => <div className="core-load-item" key={`${core.name}-${index}`}><div><strong>{core.name || `CPU ${index + 1}`}</strong><span>{core.usage_percent.toFixed(1)}%</span></div><div className="core-load-track"><span style={{ width: `${Math.max(0, Math.min(100, core.usage_percent))}%` }}/></div><small>{core.frequency_mhz ? `${core.frequency_mhz.toLocaleString()} MHz` : "Frequency not reported"}</small></div>)}</div> : <div className="history-empty">Per-core readings were not reported by the operating system.</div>}</div>
      <div className="panel"><div className="panel-header"><div><h3>Live GPU telemetry</h3><p>Read-only telemetry from NVIDIA-SMI, AMD SMI or Intel GPU Top where the vendor driver and monitoring tool expose metrics.</p></div><span className="tiny-tag">{diagnostics.gpu_telemetry.length} GPU(s)</span></div>{diagnostics.gpu_telemetry.length ? <div className="table-wrap"><table><thead><tr><th>GPU / provider</th><th>Utilization / engine activity</th><th>Memory used</th><th>Temperature</th><th>Graphics clock</th><th>Power</th></tr></thead><tbody>{diagnostics.gpu_telemetry.map((gpu, index) => <tr key={`${gpu.name}-${index}`}><td><strong>{gpu.name}</strong><br/><small>{gpu.source}</small></td><td>{gpu.utilization_percent === null ? "N/A" : `${gpu.utilization_percent.toFixed(0)}%`}</td><td>{gpu.memory_used_mib === null || gpu.memory_total_mib === null ? "N/A" : `${gpu.memory_used_mib.toFixed(0)} / ${gpu.memory_total_mib.toFixed(0)} MiB`}</td><td>{gpu.temperature_c === null ? "N/A" : `${gpu.temperature_c.toFixed(0)} °C`}</td><td>{gpu.graphics_clock_mhz === null ? "N/A" : `${gpu.graphics_clock_mhz.toFixed(0)} MHz`}</td><td>{gpu.power_draw_watts === null ? "N/A" : `${gpu.power_draw_watts.toFixed(1)} W`}</td></tr>)}</tbody></table></div> : <div className="history-empty">No supported vendor telemetry provider returned metrics. Install the appropriate vendor monitoring utility and check driver/permission support. Inventory can still be available without live telemetry.</div>}</div>
      <div className="panel"><div className="panel-header"><div><h3>Temperature and fan sensors</h3><p>Only exposed readings are listed. ACPI zones may not map to CPU/GPU sensors; requested fan speed is not actual measured RPM.</p></div><span className="tiny-tag">{diagnostics.sensors.length} reading(s)</span></div>{diagnostics.sensors.length ? <div className="table-wrap"><table><thead><tr><th>Sensor</th><th>Type</th><th>Reading</th><th>Limit</th><th>Status</th><th>Source</th></tr></thead><tbody>{diagnostics.sensors.map((sensor, index) => <tr key={`${sensor.name}-${index}`}><td>{sensor.name}</td><td>{sensor.category}</td><td className="mono">{sensor.value.toFixed(1)} {sensor.unit}</td><td>{sensor.critical_limit === null ? "Not reported" : `${sensor.critical_limit.toFixed(1)} ${sensor.unit}`}</td><td><span className={`health-status ${sensor.status === "critical" ? "status-failed" : (sensor.category === "fan" && sensor.status.toLowerCase().includes("zero rpm")) ? "status-attention" : "status-unknown"}`}>{sensor.status.replaceAll("_", " ")}</span></td><td>{sensor.source}</td></tr>)}</tbody></table></div> : <div className="history-empty">No temperature or fan data was exposed during this scan.</div>}</div>
      <div className="panel"><div className="panel-header"><div><h3>Operating-system device warnings</h3><p>Device warnings returned by the current operating-system adapter. Physical testing is not included.</p></div><span className="tiny-tag">{diagnostics.problem_devices.length} item(s)</span></div>{diagnostics.problem_devices.length ? <div className="history-list">{diagnostics.problem_devices.map((device, index) => <div className="history-row" key={`${device.instance_id}-${index}`}><span className="history-indicator warn"/><div><strong>{device.name}</strong><span>{device.class ?? "Device"} · {device.status}{device.problem_code === null ? "" : ` · Code ${device.problem_code}: ${device.problem_description ?? "Description unavailable"}`} · {device.source}</span></div></div>)}</div> : <div className="history-empty">No device warnings were returned, or this platform does not expose a comparable problem-device inventory.</div>}</div>
      <div className="panel"><div className="panel-header"><div><h3>Driver inventory</h3><p>Installed driver/device mappings returned by the operating system. Version age is not, by itself, proof of a fault.</p></div><span className="tiny-tag">{diagnostics.drivers.length} record(s)</span></div>{diagnostics.drivers.length ? <div className="table-wrap"><table><thead><tr><th>Device</th><th>Manufacturer</th><th>Driver</th><th>Version</th><th>Signed</th></tr></thead><tbody>{diagnostics.drivers.slice(0, 100).map((driver, index) => <tr key={`${driver.device}-${index}`}><td>{driver.device}</td><td>{driver.manufacturer ?? "Not reported"}</td><td>{driver.driver ?? "Not reported"}</td><td className="mono">{driver.version ?? "Not reported"}</td><td>{driver.signed === null ? "Unknown" : driver.signed ? "Yes" : "No"}</td></tr>)}</tbody></table></div> : <div className="history-empty">Driver inventory is unavailable. On Linux, install pciutils for PCI device/driver mappings.</div>}</div>
      <div className="panel"><div className="panel-header"><div><h3>USB and display connectors</h3><p>Enumerated devices and reported connector state. This does not test physical port condition.</p></div><span className="tiny-tag">{diagnostics.peripherals.length} record(s)</span></div>{diagnostics.peripherals.length ? <div className="table-wrap"><table><thead><tr><th>Category</th><th>Device / connector</th><th>Reported state</th><th>Identifier / modes</th><th>Link speed</th></tr></thead><tbody>{diagnostics.peripherals.map((item, index) => <tr key={`${item.category}-${item.name}-${index}`}><td>{item.category}</td><td>{item.name}</td><td>{item.status ?? "Not reported"}</td><td className="mono">{item.identifier ?? "Not reported"}</td><td>{item.link_speed ?? "Not reported"}</td></tr>)}</tbody></table></div> : <div className="history-empty">No USB or display connector inventory was returned.</div>}</div>
      <div className="panel recovery-guide"><div className="panel-header"><div><h3>BIOS / UEFI and boot guidance</h3><p>Model-aware guidance based on the detected PC manufacturer. Exact keys vary by model and firmware revision.</p></div><span className="tiny-tag">Guidance only</span></div><div className="recovery-guide-grid"><div><strong>Detected computer</strong><span>{snapshot ? `${snapshot.manufacturer} ${snapshot.model}` : "Manufacturer not available"}</span><strong>Common setup key</strong><span>{biosEntryHint(snapshot?.manufacturer, "setup")}</span><strong>One-time boot menu</strong><span>{biosEntryHint(snapshot?.manufacturer, "boot")}</span></div><div><strong>Before changing firmware settings</strong><ul><li>Photograph or write down the original settings.</li><li>Back up important files and make sure you have recovery information for encrypted drives.</li><li>Do not switch UEFI/Legacy or Secure Boot settings without checking the operating-system and disk boot configuration.</li><li>For a new OS, use official installation media and verify the target USB drive before writing it.</li></ul></div></div><div className="scope-note"><ShieldCheck size={16}/><span>CrossDiag currently reads firmware information and provides guidance; it does not change BIOS settings, flash firmware or create bootable USB drives.</span></div></div>
      {diagnostics.limitations.length > 0 && <div className="notice-card blue-notice"><CircleHelp size={17}/><div><strong>Scan limitations</strong>{diagnostics.limitations.map((item) => <p key={item}>{item}</p>)}</div></div>}
    </>}
    {!diagnostics && !loading && <div className="notice-card blue-notice"><AlertTriangle size={18}/><div><strong>Run the hardware scan to collect additional evidence</strong><p>The scan checks GPU inventory, supported temperatures, battery information, network adapters, device warnings and firmware details. A detected device is not proof of successful operation.</p></div></div>}
  </div>;
}

function PerformancePage({ result, loading, error, onRun }: { result: PerformanceTestResult | null; loading: string | null; error: string | null; onRun: (testId: "cpu_single_core" | "cpu_multi_core" | "memory_smoke", memoryMb?: number) => void }) {
  const [memoryMb, setMemoryMb] = useState(128);
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">BOUNDED HARDWARE TESTS</div><h2>Performance and memory checks</h2><p>Short, user-confirmed workloads. These tests detect some calculation or read-back mismatches; they are not certified CPU/GPU stress or full physical RAM tests.</p></div><span className="tiny-tag">Resource-limited</span></div>
    {error && <div className="alert alert-error"><XCircle size={17}/><div><strong>Performance test could not run</strong><span>{error}</span></div></div>}
    <div className="performance-grid">
      <article className="panel performance-card"><div className="detail-icon"><Cpu size={19}/></div><h3>Single-core CPU calculation benchmark</h3><p>Counts prime candidates for five seconds on one worker thread. Useful for a basic repeatable calculation check, not a thermal stress test.</p><ul><li>Five-second bounded run</li><li>One logical worker</li><li>Reports checks per second</li></ul><button className="primary-button" onClick={() => onRun("cpu_single_core")} disabled={loading !== null}>{loading === "cpu_single_core" ? <LoaderCircle className="spin" size={14}/> : <CirclePlay size={14}/>} {loading === "cpu_single_core" ? "Running benchmark…" : "Run single-core test"}</button></article>
      <article className="panel performance-card"><div className="detail-icon"><Activity size={19}/></div><h3>Multi-core CPU calculation benchmark</h3><p>Runs a bounded calculation workload on up to eight worker threads to compare aggregate throughput.</p><ul><li>Five-second default run</li><li>Maximum eight worker threads</li><li>Not a certified stability test</li></ul><button className="primary-button" onClick={() => onRun("cpu_multi_core")} disabled={loading !== null}>{loading === "cpu_multi_core" ? <LoaderCircle className="spin" size={14}/> : <CirclePlay size={14}/>} {loading === "cpu_multi_core" ? "Running multi-core test…" : "Run multi-core test"}</button></article>
      <article className="panel performance-card"><div className="detail-icon"><MemoryStick size={19}/></div><h3>Configurable RAM multi-pattern smoke test</h3><p>Allocates application memory and verifies six deterministic write/read-back patterns. CrossDiag refuses requests that exceed its memory-headroom guard.</p><label className="threshold-control">Test allocation<select value={memoryMb} onChange={(event) => setMemoryMb(Number(event.target.value))}><option value={64}>64 MiB — light</option><option value={128}>128 MiB — standard</option><option value={256}>256 MiB — extended</option><option value={512}>512 MiB — maximum</option></select></label><ul><li>32–512 MiB configurable allocation</li><li>Six different data patterns</li><li>Limited to a quarter of reported available memory</li></ul><button className="primary-button" onClick={() => onRun("memory_smoke", memoryMb)} disabled={loading !== null}>{loading === "memory_smoke" ? <LoaderCircle className="spin" size={14}/> : <CirclePlay size={14}/>} {loading === "memory_smoke" ? "Checking memory…" : `Run ${memoryMb} MiB RAM test`}</button></article>
    </div>
    {result && <div className="panel action-result"><div className="panel-header"><div><h3>{result.test_id === "cpu_single_core" ? "Single-core CPU result" : result.test_id === "cpu_multi_core" ? "Multi-core CPU result" : "RAM smoke-test result"}</h3><p>{dateLabel(result.started_at)} · {result.duration_ms} ms</p></div><span className={`health-status ${result.status === "passed" || result.status === "completed" ? "status-healthy" : "status-attention"}`}>{result.status}</span></div><p className="performance-summary">{result.summary}</p>{result.metric_value !== null && <div className="performance-metric"><strong>{result.metric_value.toFixed(1)}</strong><span>{result.metric_unit}</span></div>}{result.limitations.map((item) => <p className="result-note" key={item}>{item}</p>)}</div>}
    <div className="notice-card blue-notice"><ShieldCheck size={17}/><div><strong>Interpret results carefully</strong><p>A successful user-space memory smoke test does not certify every physical memory address, ECC function, DIMM timing or slot. For comprehensive RAM testing, boot a dedicated memory tester such as Memtest86+. GPU telemetry is passive monitoring, not GPU stress testing; GPU fault diagnosis still needs compatible vendor tests and corroborating evidence.</p></div></div>
  </div>;
}

function StoragePage({
  drives,
  query,
  setQuery,
  snapshot,
  storageScan,
  storageHistory,
  storageLoading,
  storageError,
  thresholdPercent,
  setThresholdPercent,
  onScan,
}: {
  drives: SystemSnapshot["drives"];
  query: string;
  setQuery: (q: string) => void;
  snapshot: SystemSnapshot | null;
  storageScan: StorageScan | null;
  storageHistory: StorageScan[];
  storageLoading: boolean;
  storageError: string | null;
  thresholdPercent: number;
  setThresholdPercent: (value: number) => void;
  onScan: () => void;
}) {
  const formatDate = (value?: string) => {
    if (!value) return "Not collected";
    const raw = value.startsWith("unix:") ? Number(value.slice(5)) * 1000 : value;
    const date = new Date(raw);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
  };
  const scanThreshold = storageScan?.threshold_percent ?? thresholdPercent;
  const statusLabel = (status: string, remaining: number | null) => {
    if (status === "failed") return "SMART failure";
    if (remaining !== null && remaining <= scanThreshold) return "Bad condition";
    const labels: Record<string, string> = {
      healthy: "SMART passed",
      attention: "Needs attention",
      unknown: "Unknown",
      unsupported: "Unsupported",
    };
    return labels[status] ?? status;
  };
  const installHelp = snapshot?.os_name.toLowerCase().includes("windows")
    ? "Windows: install with: winget install --id smartmontools.smartmontools -e. Then reopen CrossDiag and verify smartctl --version in a terminal."
    : "Linux: install smartmontools with your distribution package manager: Ubuntu/Debian: sudo apt install smartmontools · Fedora: sudo dnf install smartmontools · Arch: sudo pacman -S smartmontools. Some devices require root permissions for SMART access.";

  return <div className="stack-layout">
    <div className="storage-toolbar panel">
      <div>
        <div className="eyebrow">READ-ONLY DRIVE ASSESSMENT</div>
        <h2>Physical storage health</h2>
        <p>Collect SMART evidence from supported HDDs and SSDs. No self-tests, writes, repairs or firmware changes are started.</p>
      </div>
      <div className="storage-toolbar-actions">
        <label className="threshold-control">Threshold for next scan
          <select value={thresholdPercent} onChange={(event) => setThresholdPercent(Number(event.target.value))}>
            {[80, 85, 90, 95].map((value) => <option key={value} value={value}>{value}%</option>)}
          </select>
        </label>
        <button className="primary-button" onClick={onScan} disabled={storageLoading}>
          {storageLoading ? <LoaderCircle className="spin" size={15}/> : <Activity size={15}/>}
          {storageLoading ? "Scanning drives…" : "Scan SMART health"}
        </button>
      </div>
    </div>

    {storageError && <div className="alert alert-error"><AlertTriangle size={17}/><div><strong>Storage scan needs attention</strong><span>{storageError}</span></div></div>}

    {!storageScan && !storageLoading && <div className="notice-card blue-notice"><HardDrive size={18}/><div><strong>SMART scan has not been run in this session</strong><p>Select “Scan SMART health” to enumerate physical drives and read supported drive-health information. The scan requires smartmontools to be installed on this computer.</p></div></div>}

    {storageScan && <>
      <div className="storage-summary-grid">
        <div className="metric-card accent-blue"><div className="metric-top"><span>Physical drives found</span><HardDrive size={17}/></div><div className="metric-value">{storageScan.disks.length}</div><div className="metric-foot">Reported by smartmontools</div></div>
        <div className="metric-card accent-cyan"><div className="metric-top"><span>SMART tool</span><CheckCircle2 size={17}/></div><div className="metric-value small-metric">{storageScan.smartctl_available ? "Available" : "Missing"}</div><div className="metric-foot">{storageScan.smartctl_version ?? "smartctl not detected"}</div></div>
        <div className="metric-card accent-amber"><div className="metric-top"><span>Scan outcome</span><Activity size={17}/></div><div className="metric-value small-metric">{storageScan.scan_status.replaceAll("_", " ")}</div><div className="metric-foot">{formatDate(storageScan.scanned_at)}</div></div>
      </div>

      {!storageScan.smartctl_available && <div className="panel install-panel"><div className="install-panel-icon"><BookOpenText size={20}/></div><div><strong>Install smartmontools to enable SMART diagnostics</strong><p>{installHelp}</p><a className="external-link" href="https://www.smartmontools.org/" target="_blank" rel="noreferrer">Official smartmontools project <ExternalLink size={13}/></a></div></div>}

      {storageScan.limitations.length > 0 && <div className="notice-card blue-notice"><CircleHelp size={17}/><div><strong>Scan notes and limitations</strong>{storageScan.limitations.map((item) => <p key={item}>{item}</p>)}</div></div>}

      {storageScan.disks.length > 0 ? <div className="storage-disk-list">{storageScan.disks.map((disk) => <article className="panel disk-health-card" key={disk.device}>
        <div className="disk-health-header">
          <div className="disk-heading-icon"><HardDrive size={20}/></div>
          <div className="disk-heading-copy"><span className="eyebrow">{disk.drive_kind} · {disk.protocol ?? "Protocol not reported"}</span><h3>{disk.model ?? disk.device}</h3><p className="mono">{disk.device}{disk.device_type ? ` · device type ${disk.device_type}` : ""}</p></div>
          <span className={`health-status status-${disk.status}`}>{statusLabel(disk.status, disk.remaining_endurance_percent)}</span>
        </div>
        <p className="disk-reason">{disk.status_reason}</p>
        {disk.findings.length > 0 && <div className="disk-findings-panel"><strong>Detected findings</strong><ul>{disk.findings.map((finding) => <li key={finding}>{finding}</li>)}</ul></div>}
        <div className="disk-metric-grid">
          <DiskMetric label="Capacity" value={disk.capacity_bytes ? formatBytes(disk.capacity_bytes) : "Not reported"}/>
          <DiskMetric label="SMART overall" value={disk.smart_passed === null ? "Not reported" : disk.smart_passed ? "Passed" : "Failed"}/>
          <DiskMetric label="Temperature" value={disk.temperature_c === null ? "Not reported" : `${disk.temperature_c} °C`}/>
          <DiskMetric label="Power-on hours" value={disk.power_on_hours === null ? "Not reported" : `${disk.power_on_hours.toLocaleString()} h`}/>
          <DiskMetric label="Media errors" value={disk.media_errors === null ? "Not reported" : disk.media_errors.toLocaleString()}/>
          <DiskMetric label="Available spare" value={disk.available_spare_percent === null ? "Not reported" : `${disk.available_spare_percent}%`}/>
        </div>
        <div className="endurance-block">
          <div className="endurance-heading"><div><strong>Estimated remaining endurance</strong><span>{disk.endurance_basis ?? "No standardized remaining-life value reported"}</span></div><strong className={disk.remaining_endurance_percent !== null && disk.remaining_endurance_percent <= scanThreshold ? "warning-number" : ""}>{disk.remaining_endurance_percent === null ? "Not available" : `${Number.isInteger(disk.remaining_endurance_percent) ? disk.remaining_endurance_percent.toFixed(0) : disk.remaining_endurance_percent.toFixed(1)}%`}</strong></div>
          {disk.remaining_endurance_percent !== null && <><div className="endurance-track"><div style={{ width: `${Math.max(0, Math.min(100, disk.remaining_endurance_percent))}%` }}/></div><span className="threshold-caption">Scan policy: greater than {storageScan.threshold_percent}% = OK · {storageScan.threshold_percent}% or below = Bad condition. This is a policy flag, not proof of immediate drive failure.</span></>}
          {disk.endurance_used_percent !== null && <span className="threshold-caption">NVMe endurance used: {disk.endurance_used_percent.toFixed(1)}% (hardware-reported endurance used).</span>}
        </div>
        <div className="disk-findings-grid">
          <DiskMetric label="Reallocated sectors" value={disk.reallocated_sectors === null ? "Not reported" : disk.reallocated_sectors.toLocaleString()}/>
          <DiskMetric label="Pending sectors" value={disk.current_pending_sectors === null ? "Not reported" : disk.current_pending_sectors.toLocaleString()}/>
          <DiskMetric label="Uncorrectable sectors" value={disk.offline_uncorrectable === null ? "Not reported" : disk.offline_uncorrectable.toLocaleString()}/>
          <DiskMetric label="NVMe critical warning" value={disk.critical_warning === null ? "Not reported" : `0x${disk.critical_warning.toString(16).toUpperCase()}`}/>
        </div>
        {disk.smart_attributes.length > 0 && <details className="smart-details"><summary>Show {disk.smart_attributes.length} raw SMART attributes</summary><div className="table-wrap"><table><thead><tr><th>ID</th><th>Attribute</th><th>Value</th><th>Worst</th><th>Threshold</th><th>Raw</th><th>Failed</th></tr></thead><tbody>{disk.smart_attributes.map((attribute, index) => <tr key={`${attribute.id ?? "x"}-${attribute.name}-${index}`}><td>{attribute.id ?? "—"}</td><td>{attribute.name}</td><td>{attribute.normalized_value ?? "—"}</td><td>{attribute.worst_value ?? "—"}</td><td>{attribute.threshold ?? "—"}</td><td>{attribute.raw_value ?? "—"}</td><td>{attribute.when_failed ?? "—"}</td></tr>)}</tbody></table></div></details>}
        {disk.limitations.length > 0 && <details className="smart-details"><summary>Evidence limitations</summary><ul>{disk.limitations.map((item) => <li key={item}>{item}</li>)}</ul></details>}
      </article>)}</div> : storageScan.smartctl_available && <EmptyState icon={HardDrive} title="No SMART-capable physical drives enumerated" description="The OS may be hiding devices behind a RAID controller, USB bridge or driver. Check permissions and supported controller options; this result does not mean the PC has no storage."/>}
    </>}

    <div className="panel">
      <div className="panel-header"><div><h3>Mounted volumes</h3><p>{snapshot ? `${snapshot.drives.length} filesystem volume(s) reported by the OS` : "Collect system inventory to see mounted volumes"}</p></div><div className="search-field"><Search size={15}/><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Filter volumes…"/></div></div>
      {drives.length ? <div className="table-wrap"><table><thead><tr><th>Volume</th><th>Mount point</th><th>Filesystem</th><th>Total</th><th>Available</th><th>Free space</th></tr></thead><tbody>{drives.map((drive) => { const freePct = drive.total_bytes > 0 ? Math.round(drive.available_bytes / drive.total_bytes * 100) : 0; return <tr key={`${drive.mount_point}-${drive.name}`}><td><span className="table-device"><HardDrive size={16}/>{drive.name || "Unnamed volume"}{drive.removable && <span className="tiny-tag">Removable</span>}</span></td><td className="mono">{drive.mount_point}</td><td>{drive.file_system || "Unknown"}</td><td>{formatBytes(drive.total_bytes)}</td><td>{formatBytes(drive.available_bytes)}</td><td><div className="free-space"><div className="mini-track"><div style={{ width: `${freePct}%` }}/></div><span>{freePct}%</span></div></td></tr>; })}</tbody></table></div> : <EmptyState icon={HardDrive} title="No mounted volumes to display" description={snapshot ? "No mounted volumes were returned by the OS or your filter has no matches." : "Run the desktop app and refresh inventory."}/>}
    </div>

    <div className="panel history-panel"><div className="panel-header"><div><h3>Recent storage scans</h3><p>Local scan history · last 100 scans retained</p></div><span className="tiny-tag">{storageHistory.length} saved</span></div>{storageHistory.length ? <div className="history-list">{[...storageHistory].reverse().slice(0, 8).map((history, index) => <div className="history-row" key={`${history.scanned_at}-${index}`}><span className={`history-indicator ${history.disks.some((disk) => disk.status === "failed") ? "bad" : history.disks.some((disk) => disk.status === "attention") ? "warn" : history.scan_status !== "completed" || history.disks.some((disk) => disk.status === "unknown") ? "unknown" : "good"}`}/><div><strong>{formatDate(history.scanned_at)}</strong><span>{history.disks.length} physical drive(s) · SMART {history.scan_status.replaceAll("_", " ")}</span></div><span className="history-threshold">Threshold {history.threshold_percent}%</span></div>)}</div> : <div className="history-empty">No storage scan has been saved yet.</div>}</div>

    <StorageTrendPanel scans={storageHistory}/>

    <div className="small-print"><CircleHelp size={14}/> SMART indicators are predictive, vendor-specific and incomplete. Back up important data. A passing SMART result is not a guarantee; physical ports, cables, power supplies and intermittent faults require additional tests.</div>
  </div>;
}

function DiskMetric({ label, value }: { label: string; value: string }) {
  return <div className="disk-metric"><span>{label}</span><strong>{value}</strong></div>;
}

function DetailCard({ icon: Icon, title, rows }: { icon: LucideIcon; title: string; rows: [string, string | undefined][] }) {
  return <div className="panel detail-card"><div className="detail-heading"><div className="detail-icon"><Icon size={17}/></div><h3>{title}</h3></div><div className="detail-rows">{rows.map(([key, value]) => <div className="detail-row" key={key}><span>{key}</span><strong title={value ?? "Not reported"}>{value || "Not reported"}</strong></div>)}</div></div>;
}
function UpdateCenterPage({ status, loading, error, installing, installResult, onCheck, onOpenOsUpdates, onInstallOsUpdates, onInstall }: {
  status: UpdateStatus | null;
  loading: boolean;
  error: string | null;
  installing: string | null;
  installResult: UpdateInstallResult | null;
  onCheck: () => void;
  onOpenOsUpdates: () => void;
  onInstallOsUpdates: () => void;
  onInstall: (update: UpdateStatus["updates"][number]) => void;
}) {
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">OS + APPLICATION MAINTENANCE</div><h2>Update center</h2><p>Discover available updates, inspect their source and selectively install supported application/package updates. Windows Update Agent entries remain managed through Windows Settings.</p></div><div className="toolbar-buttons">{status?.platform === "Windows" && <button className="subtle-button" onClick={onOpenOsUpdates}><ExternalLink size={14}/> Windows Update settings</button>}{status?.platform === "Linux" && status.updates.some((update) => ["APT simulation", "DNF check-update", "Pacman -Qu", "Zypper"].includes(update.source)) && <button className="warning-button" onClick={onInstallOsUpdates} disabled={installing !== null}>{installing === "__system_updates__" ? <LoaderCircle className="spin" size={14}/> : <Download size={14}/>} Install distribution updates</button>}<button className="primary-button" onClick={onCheck} disabled={loading}>{loading ? <LoaderCircle className="spin" size={15}/> : <RefreshCw size={15}/>} {loading ? "Checking…" : "Check for updates"}</button></div></div>
    {error && <div className="alert alert-error"><XCircle size={17}/><div><strong>Update operation needs attention</strong><span>{error}</span></div></div>}
    {!status && <div className="notice-card blue-notice"><ArrowDownToLine size={18}/><div><strong>No update check has been run</strong><p>Run a read-only discovery pass. Package metadata may be stale; update installation is a separate, user-confirmed action.</p></div></div>}
    {status && <>
      <div className="storage-summary-grid"><div className="metric-card accent-blue"><div className="metric-top"><span>Update status</span><Activity size={17}/></div><div className="metric-value small-metric">{status.status.replaceAll("_", " ")}</div><div className="metric-foot">{status.platform}</div></div><div className="metric-card accent-cyan"><div className="metric-top"><span>Available updates</span><ArrowDownToLine size={17}/></div><div className="metric-value">{status.update_count === null ? "—" : status.update_count}</div><div className="metric-foot">Reported by {status.package_manager}</div></div><div className="metric-card accent-amber"><div className="metric-top"><span>Last check</span><RefreshCw size={17}/></div><div className="metric-value small-metric">{dateLabel(status.checked_at)}</div><div className="metric-foot">Discovery only</div></div></div>
      <div className="panel"><div className="panel-header"><div><h3>Available updates</h3><p>{status.updates.length} item(s) listed · install only after individual approval</p></div><span className="tiny-tag">{status.package_manager}</span></div>{status.updates.length ? <div className="update-list">{status.updates.map((update, index) => <div className="update-row" key={`${update.name}-${update.source}-${index}`}><div className="update-main"><span className="history-indicator warn"/><div><strong>{update.name}</strong><span>{[update.version, update.source, update.detail].filter(Boolean).join(" · ")}</span></div></div>{update.install_id ? <button className="small-action" disabled={installing !== null} onClick={() => onInstall(update)}>{installing === update.install_id ? <LoaderCircle className="spin" size={13}/> : <Download size={13}/>} {installing === update.install_id ? "Installing…" : "Install"}</button> : <span className="tiny-tag">OS-managed</span>}</div>)}</div> : <div className="history-empty">No pending packages were listed by this check, or the platform returned no update entries.</div>}</div>
      {installResult && <div className={`panel action-result ${installResult.success ? "result-success" : "result-warning"}`}><div className="panel-header"><div><h3>{installResult.success ? "Update command completed" : "Update command failed"}</h3><p>{installResult.package} · {installResult.source} · exit code {installResult.exit_code ?? "unknown"}</p></div><span className={`health-status ${installResult.success ? "status-healthy" : "status-attention"}`}>{installResult.success ? "Completed" : "Needs review"}</span></div><pre className="command-output">{installResult.output || "The installer returned no textual output."}</pre>{installResult.limitations.map((item) => <p className="result-note" key={item}>{item}</p>)}</div>}
      {status.limitations.length > 0 && <div className="notice-card blue-notice"><CircleHelp size={17}/><div><strong>Update-check notes</strong>{status.limitations.map((item) => <p key={item}>{item}</p>)}</div></div>}
    </>}
    <div className="scope-note"><ShieldCheck size={16}/><span>CrossDiag installs one selected package only after confirmation. Major operating-system updates and firmware remain in their official OS/vendor workflows. Save work and back up important files before installing updates.</span></div>
  </div>;
}

function FaultAnalyzerPage({ report, loading, error, onScan }: { report: FaultReport | null; loading: boolean; error: string | null; onScan: () => void }) {
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">SYSTEM LOG EVIDENCE</div><h2>Fault and error analyzer</h2><p>Review recent critical/error events from Windows System logs or the current Linux boot journal. Events are evidence, not automatic hardware-failure diagnoses.</p></div><button className="primary-button" onClick={onScan} disabled={loading}>{loading ? <LoaderCircle className="spin" size={15}/> : <Search size={15}/>} {loading ? "Analyzing logs…" : "Analyze system logs"}</button></div>
    {error && <div className="alert alert-error"><XCircle size={17}/><div><strong>Log analysis failed</strong><span>{error}</span></div></div>}
    {!report && <div className="notice-card blue-notice"><Wrench size={18}/><div><strong>Analysis has not been run</strong><p>CrossDiag will read a bounded set of recent system events. Access may be limited by operating-system permissions or log retention settings.</p></div></div>}
    {report && <>
      <div className="storage-summary-grid"><div className="metric-card accent-blue"><div className="metric-top"><span>Events collected</span><AlertTriangle size={17}/></div><div className="metric-value">{report.faults.length}</div><div className="metric-foot">From {report.source}</div></div><div className="metric-card accent-cyan"><div className="metric-top"><span>Platform</span><Laptop size={17}/></div><div className="metric-value small-metric">{report.platform}</div><div className="metric-foot">Local log analysis</div></div><div className="metric-card accent-amber"><div className="metric-top"><span>Scan time</span><Activity size={17}/></div><div className="metric-value small-metric">{dateLabel(report.scanned_at)}</div><div className="metric-foot">No repairs performed</div></div></div>
      <div className="panel"><div className="panel-header"><div><h3>Recent system errors</h3><p>Latest entries returned by the operating system</p></div><span className="tiny-tag">{report.faults.length} events</span></div>{report.faults.length ? <div className="fault-list">{report.faults.map((fault, index) => <article className="fault-item" key={`${fault.timestamp}-${fault.source}-${index}`}><div className="fault-meta"><span className="health-status status-attention">{fault.severity}</span><span>{fault.source}</span><span>{fault.code ? `Code ${fault.code}` : "No event ID"}</span><time>{dateLabel(fault.timestamp ?? undefined)}</time></div><p>{fault.message}</p></article>)}</div> : <div className="history-empty">No matching critical/error entries were returned by this bounded query. This does not prove the system has no faults.</div>}</div>
      {report.limitations.length > 0 && <div className="notice-card blue-notice"><CircleHelp size={17}/><div><strong>Analysis limitations</strong>{report.limitations.map((item) => <p key={item}>{item}</p>)}</div></div>}
    </>}
  </div>;
}

function DriversPage({ diagnostics, snapshot, loading, notice, onScan, onOpenManager }: { diagnostics: HardwareDiagnostics | null; snapshot: SystemSnapshot | null; loading: boolean; notice: string | null; onScan: () => void; onOpenManager: () => void }) {
  const brand = (snapshot?.manufacturer ?? "").toLowerCase();
  const support = diagnostics?.vendor_support.support_url ?? (brand.includes("intel") ? "https://www.intel.com/content/www/us/en/support/detect.html" : null);
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">DEVICE + DRIVER MAINTENANCE</div><h2>Drivers and devices</h2><p>Review driver mappings, signatures and problem-device states. CrossDiag does not automatically replace drivers or install unsigned packages.</p></div><div className="toolbar-buttons"><button className="subtle-button" onClick={onOpenManager}><ExternalLink size={14}/> Open system device manager</button><button className="primary-button" onClick={onScan} disabled={loading}>{loading ? <LoaderCircle className="spin" size={15}/> : <RefreshCw size={15}/>} Refresh inventory</button></div></div>
    {notice && <div className="notice-card blue-notice"><CircleHelp size={17}/><div><strong>System driver utility</strong><p>{notice}</p></div></div>}
    <div className="storage-summary-grid"><div className="metric-card accent-blue"><div className="metric-top"><span>Driver records</span><Cpu size={17}/></div><div className="metric-value">{diagnostics?.drivers.length ?? "—"}</div><div className="metric-foot">Reported by the OS</div></div><div className="metric-card accent-amber"><div className="metric-top"><span>Problem devices</span><AlertTriangle size={17}/></div><div className="metric-value">{diagnostics?.problem_devices.length ?? "—"}</div><div className="metric-foot">Device status is not a root-cause diagnosis</div></div><div className="metric-card accent-cyan"><div className="metric-top"><span>Detected PC</span><Laptop size={17}/></div><div className="metric-value small-metric">{snapshot?.manufacturer ?? "Unknown"}</div><div className="metric-foot">{snapshot?.model ?? "Model not reported"}</div></div></div>
    {diagnostics?.vendor_support && <div className="panel vendor-support-panel"><div className="panel-header"><div><h3>{diagnostics.vendor_support.vendor} support profile</h3><p>{diagnostics.vendor_support.product_family ? `Detected product family: ${diagnostics.vendor_support.product_family}` : `Detected model: ${snapshot?.model ?? "Not reported"}`}</p></div><span className="tiny-tag">Manufacturer match</span></div><div className="detail-grid"><DetailCard icon={Wrench} title="Recommended diagnostic utility" rows={[["Utility", diagnostics.vendor_support.diagnostic_utility ?? "No vendor utility identified"], ["Firmware setup", diagnostics.vendor_support.firmware_setup_hint], ["One-time boot menu", diagnostics.vendor_support.boot_menu_hint]]}/><DetailCard icon={ShieldCheck} title="Compatibility guidance" rows={[["Manufacturer", diagnostics.vendor_support.vendor], ["Model", snapshot?.model ?? "Not reported"], ["Guidance", diagnostics.vendor_support.notes]]}/></div>{support && <a className="external-link" href={support ?? undefined} target="_blank" rel="noreferrer">Open official manufacturer support <ExternalLink size={13}/></a>}</div>}
    {diagnostics?.driver_findings.length ? <div className="panel"><div className="panel-header"><div><h3>Driver compatibility findings</h3><p>Only explicit operating-system evidence is flagged. CrossDiag does not infer that a driver is outdated from its age.</p></div><span className="tiny-tag">{diagnostics.driver_findings.length} finding(s)</span></div><div className="history-list">{diagnostics.driver_findings.map((finding, index) => <div className="history-row" key={`${finding.code}-${finding.device}-${index}`}><span className={`history-indicator ${finding.severity === "warning" ? "warn" : ""}`}/><div><strong>{finding.device} · {finding.code.replaceAll("_", " ")}</strong><span>{finding.explanation}</span><small>{finding.recommendation}</small></div></div>)}</div></div> : null}
    <div className="panel"><div className="panel-header"><div><h3>Installed driver inventory</h3><p>Version age alone does not prove a driver fault.</p></div><span className="tiny-tag">{diagnostics?.drivers.length ?? 0} records</span></div>{diagnostics?.drivers.length ? <div className="table-wrap"><table><thead><tr><th>Device</th><th>Manufacturer</th><th>Driver</th><th>Version</th><th>Signed</th></tr></thead><tbody>{diagnostics.drivers.slice(0, 150).map((driver, index) => <tr key={`${driver.device}-${index}`}><td>{driver.device}</td><td>{driver.manufacturer ?? "Not reported"}</td><td>{driver.driver ?? "Not reported"}</td><td className="mono">{driver.version ?? "Not reported"}</td><td>{driver.signed === null ? "Unknown" : driver.signed ? "Yes" : "No"}</td></tr>)}</tbody></table></div> : <div className="history-empty">Collect hardware diagnostics to populate driver inventory.</div>}</div>
    <div className="panel"><div className="panel-header"><div><h3>Problem devices</h3><p>Devices the operating system reports as not functioning normally.</p></div><span className="tiny-tag">{diagnostics?.problem_devices.length ?? 0} findings</span></div>{diagnostics?.problem_devices.length ? <div className="history-list">{diagnostics.problem_devices.map((device, index) => <div className="history-row" key={`${device.name}-${index}`}><span className="history-indicator bad"/><div><strong>{device.name}</strong><span>{device.class ?? "Unknown class"} · {device.status}{device.problem_code === null ? "" : ` · Code ${device.problem_code}: ${device.problem_description ?? "Description unavailable"}`} · {device.instance_id ?? "No device ID"}</span></div></div>)}</div> : <div className="history-empty">No problem devices were returned, or the hardware scan has not yet been run.</div>}</div>
    <div className="notice-card blue-notice"><ShieldCheck size={17}/><div><strong>Official driver source</strong><p>Use the computer manufacturer's official support site for model-specific chipset, graphics, network and firmware packages. CrossDiag does not recommend third-party driver download aggregators.</p><a className="external-link" href={support ?? undefined} target="_blank" rel="noreferrer">Open detected vendor support page <ExternalLink size={13}/></a></div></div>
  </div>;
}

function RepairCenterPage({ catalog, result, running, error, onRun }: { catalog: RepairAction[]; result: RepairResult | null; running: string | null; error: string | null; onRun: (action: RepairAction) => void }) {
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">SAFE MAINTENANCE WORKFLOWS</div><h2>Repair center</h2><p>Run narrowly scoped, allow-listed checks and maintenance actions. Every action is displayed before execution and must be confirmed.</p></div><span className="health-status status-healthy">No unattended repairs</span></div>
    {error && <div className="alert alert-error"><XCircle size={17}/><div><strong>Action could not run</strong><span>{error}</span></div></div>}
    <div className="notice-card blue-notice"><ShieldCheck size={17}/><div><strong>Safety boundary</strong><p>Available actions are limited to DNS cache flushing and read-only integrity/service/package checks. CrossDiag does not reset the network stack, remove drivers, modify boot configuration or repair files automatically.</p></div></div>
    <div className="repair-action-grid">{catalog.map((action) => <article className="panel repair-action" key={action.id}><div className="repair-action-top"><span className="tiny-tag">{action.category}</span><span className={`health-status ${action.is_read_only ? "status-healthy" : "status-attention"}`}>{action.is_read_only ? "Read-only" : "Changes cache"}</span></div><h3>{action.title}</h3><p>{action.description}</p><div className="repair-action-bottom"><span>{action.requires_admin ? "Administrator access may be required" : "Standard permissions usually sufficient"}</span><button className="primary-button" onClick={() => onRun(action)} disabled={running !== null}>{running === action.id ? <LoaderCircle className="spin" size={14}/> : <CirclePlay size={14}/>} {running === action.id ? "Running…" : "Run action"}</button></div></article>)}</div>
    {result && <div className="panel action-result"><div className="panel-header"><div><h3>{result.title}</h3><p>Started {dateLabel(result.started_at)} · finished {dateLabel(result.completed_at)}</p></div><span className={`health-status ${result.success ? "status-healthy" : "status-attention"}`}>{result.success ? "Command succeeded" : `Exit ${result.exit_code ?? "unknown"}`}</span></div><pre className="command-output">{result.output || "The command returned no textual output."}</pre>{result.limitations.map((item) => <p className="result-note" key={item}>{item}</p>)}</div>}
  </div>;
}

function RecoveryPage({ manufacturer }: { manufacturer: string }) {
  const brand = manufacturer.toLowerCase();
  const setupKey = brand.includes("dell") ? "F2" : brand.includes("hp") || brand.includes("hewlett") ? "Esc, then F10" : brand.includes("lenovo") ? "F1 or F2; Novo button on supported models" : brand.includes("asus") ? "F2" : brand.includes("acer") ? "F2" : brand.includes("msi") ? "Delete" : "F2, Delete or Esc (model-dependent)";
  const bootKey = brand.includes("dell") ? "F12" : brand.includes("hp") || brand.includes("hewlett") ? "Esc, then F9" : brand.includes("lenovo") ? "F12 or Novo button on supported models" : brand.includes("asus") ? "Esc" : brand.includes("acer") ? "F12 (may need enabling in firmware)" : brand.includes("msi") ? "F11" : "F12, F11, Esc or a vendor-specific key";
  const [imagePath, setImagePath] = useState("");
  const [expectedHash, setExpectedHash] = useState("");
  const [imageInfo, setImageInfo] = useState<RecoveryImageInfo | null>(null);
  const [verifyError, setVerifyError] = useState<string | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [media, setMedia] = useState<RecoveryMediaTarget[]>([]);
  const [mediaLoading, setMediaLoading] = useState(false);
  const [mediaError, setMediaError] = useState<string | null>(null);
  const verifyImage = async () => {
    setVerifying(true); setVerifyError(null); setImageInfo(null);
    try { setImageInfo(await inspectRecoveryImage(imagePath, expectedHash || undefined)); }
    catch (reason) { setVerifyError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setVerifying(false); }
  };
  const refreshMedia = async () => {
    setMediaLoading(true); setMediaError(null);
    try { setMedia(await listRecoveryMedia()); }
    catch (reason) { setMediaError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setMediaLoading(false); }
  };
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">FIRMWARE + OPERATING SYSTEM RECOVERY</div><h2>BIOS, UEFI & recovery assistant</h2><p>Verify local installation images, inspect candidate USB drives and follow a safe media-preparation workflow. CrossDiag does not write to USB devices.</p></div><span className="tiny-tag">Guided workflow</span></div>
    <div className="storage-summary-grid"><div className="metric-card accent-blue"><div className="metric-top"><span>Detected manufacturer</span><Laptop size={17}/></div><div className="metric-value small-metric">{manufacturer || "Unknown"}</div><div className="metric-foot">Keys vary by exact model</div></div><div className="metric-card accent-cyan"><div className="metric-top"><span>Firmware setup key</span><Settings2 size={17}/></div><div className="metric-value small-metric">{setupKey}</div><div className="metric-foot">Press during startup</div></div><div className="metric-card accent-violet"><div className="metric-top"><span>One-time boot menu</span><HardDriveDownload size={17}/></div><div className="metric-value small-metric">{bootKey}</div><div className="metric-foot">Consult the exact model manual</div></div></div>
    <div className="panel recovery-guide"><div className="panel-header"><div><h3>Verify an OS installation image</h3><p>CrossDiag calculates SHA-256 locally. Compare it with the checksum published by the official OS publisher.</p></div><span className="tiny-tag">Read-only file check</span></div>
      <div className="image-verify-form"><label>Local ISO / IMG path<input value={imagePath} onChange={(event) => setImagePath(event.target.value)} placeholder={navigator.userAgent.includes("Windows") ? "C:\\Users\\you\\Downloads\\ubuntu.iso" : "/home/you/Downloads/ubuntu.iso"} /></label><label>Expected SHA-256 (optional)<input value={expectedHash} onChange={(event) => setExpectedHash(event.target.value)} maxLength={64} placeholder="64 hexadecimal characters from the official publisher" /></label><button className="primary-button" onClick={() => void verifyImage()} disabled={verifying || !imagePath.trim()}>{verifying ? <LoaderCircle className="spin" size={14}/> : <ShieldCheck size={14}/>} {verifying ? "Calculating SHA-256…" : "Verify image"}</button></div>
      {verifyError && <div className="alert alert-error"><XCircle size={16}/><div><strong>Image verification failed</strong><span>{verifyError}</span></div></div>}
      {imageInfo && <div className={`image-check-result ${imageInfo.checksum_matches === false ? "result-warning" : "result-success"}`}><div className="panel-header"><div><h3>{imageInfo.file_name}</h3><p>{formatBytes(imageInfo.size_bytes)} · inspected {dateLabel(imageInfo.inspected_at)}</p></div><span className={`health-status ${imageInfo.checksum_matches === false ? "status-attention" : imageInfo.checksum_matches === true ? "status-healthy" : "status-unknown"}`}>{imageInfo.checksum_matches === true ? "Checksum matches" : imageInfo.checksum_matches === false ? "Checksum mismatch" : "Hash calculated"}</span></div><div className="hash-block"><span>SHA-256</span><code>{imageInfo.sha256}</code></div>{imageInfo.limitations.map((item) => <p className="result-note" key={item}>{item}</p>)}</div>}
    </div>
    <div className="panel recovery-guide"><div className="panel-header"><div><h3>Inspect removable USB media</h3><p>Inventory only. This does not select, format, mount, unmount or write to any drive.</p></div><button className="subtle-button" onClick={() => void refreshMedia()} disabled={mediaLoading}>{mediaLoading ? <LoaderCircle className="spin" size={14}/> : <RefreshCw size={14}/>} {mediaLoading ? "Scanning drives…" : "Scan removable drives"}</button></div>
      {mediaError && <div className="alert alert-error"><XCircle size={16}/><div><strong>Could not inventory removable drives</strong><span>{mediaError}</span></div></div>}
      {media.length > 0 ? <div className="media-target-list">{media.map((device) => <div className="media-target" key={device.path}><div><strong>{device.name || "Removable storage"}</strong><span>{device.path} · {formatBytes(device.size_bytes)} · {device.transport ?? "Transport unknown"}</span><span>Mounted: {device.mounted ? "Yes" : "No / not reported"} · System disk: {device.system_disk === null ? "Unknown" : device.system_disk ? "Yes" : "No"} · Boot disk: {device.boot_disk === null ? "Unknown" : device.boot_disk ? "Yes" : "No"}</span></div><span className={`health-status ${device.system_disk || device.boot_disk ? "status-attention" : device.mounted ? "status-unknown" : "status-healthy"}`}>{device.system_disk || device.boot_disk ? "Do not target" : device.mounted ? "Mounted — verify first" : "Candidate only"}</span></div>)}</div> : <div className="history-empty">{mediaLoading ? "Checking storage devices…" : "No removable USB disk has been inventoried yet. Connect a USB drive and scan again, or use the operating system's media tool."}</div>}
      <div className="scope-note"><AlertTriangle size={16}/><span>Never assume a removable USB disk is safe to overwrite. CrossDiag intentionally does not offer a raw-device writer in this release. Preparing media with third-party tools erases the selected USB; verify the device name, model and capacity in that tool before continuing.</span></div>
    </div>
    <div className="panel recovery-guide"><div className="panel-header"><div><h3>Official downloads and media tools</h3><p>Use official images and trusted USB creation tools.</p></div><span className="tiny-tag">External tools</span></div><div className="recovery-links"><a href="https://www.microsoft.com/software-download/windows11" target="_blank" rel="noreferrer">Windows 11 download <ExternalLink size={13}/></a><a href="https://ubuntu.com/download/desktop" target="_blank" rel="noreferrer">Ubuntu download <ExternalLink size={13}/></a><a href="https://fedoraproject.org/workstation/download" target="_blank" rel="noreferrer">Fedora download <ExternalLink size={13}/></a><a href="https://rufus.ie/" target="_blank" rel="noreferrer">Rufus <ExternalLink size={13}/></a><a href="https://www.ventoy.net/" target="_blank" rel="noreferrer">Ventoy <ExternalLink size={13}/></a><a href="https://etcher.balena.io/" target="_blank" rel="noreferrer">balenaEtcher <ExternalLink size={13}/></a></div></div>
    <div className="notice-card blue-notice"><CircleHelp size={17}/><div><strong>Firmware and boot safety</strong><p>Keys are model-dependent hints. Verify them against the exact PC manual. Changing Secure Boot, UEFI/Legacy mode, TPM or boot order can trigger disk-encryption recovery or make an OS unbootable. CrossDiag does not flash firmware, change BIOS settings or install an OS.</p></div></div>
  </div>;
}

function CasesPage({ cases, error, schedule, onCreate, onUpdate, onDelete, onSchedule }: { cases: TechnicianCase[]; error: string | null; schedule: ScanSchedule; onCreate: (title: string, device: string, notes: string) => Promise<boolean>; onUpdate: (id: number, status: string, notes: string) => void; onDelete: (id: number) => void; onSchedule: (enabled: boolean, hours: number) => void }) {
  const [title, setTitle] = useState("");
  const [device, setDevice] = useState("");
  const [notes, setNotes] = useState("");
  const [selectedStatus, setSelectedStatus] = useState<Record<number, string>>({});
  const [selectedNotes, setSelectedNotes] = useState<Record<number, string>>({});
  const submit = async (event: FormEvent) => { event.preventDefault(); if (!title.trim()) return; const saved = await onCreate(title, device, notes); if (saved) { setTitle(""); setDevice(""); setNotes(""); } };
  return <div className="stack-layout">
    <div className="storage-toolbar panel"><div><div className="eyebrow">LOCAL TECHNICIAN WORKSPACE</div><h2>Cases & scheduled checks</h2><p>Keep repair notes and case status locally. Case data remains on this computer in an SQLite database.</p></div><span className="tiny-tag">Offline-first</span></div>
    {error && <div className="alert alert-error"><XCircle size={17}/><div><strong>Local workspace issue</strong><span>{error}</span></div></div>}
    <div className="panel schedule-panel"><div className="panel-header"><div><h3><CalendarClock size={16}/> Scheduled storage checks</h3><p>The native scheduler runs in the background while CrossDiag is running. It stops after the application exits.</p></div><span className={`health-status ${schedule.enabled ? "status-healthy" : "status-unknown"}`}>{schedule.enabled ? "Enabled" : "Disabled"}</span></div><div className="schedule-controls"><label className="toggle-row"><input type="checkbox" checked={schedule.enabled} onChange={(event) => onSchedule(event.target.checked, schedule.interval_hours)}/> Enable periodic SMART storage scans</label><label className="threshold-control">Run interval<select value={schedule.interval_hours} onChange={(event) => onSchedule(schedule.enabled, Number(event.target.value))}><option value={1}>Every hour</option><option value={6}>Every 6 hours</option><option value={12}>Every 12 hours</option><option value={24}>Every 24 hours</option><option value={48}>Every 48 hours</option><option value={168}>Every 7 days</option></select></label><span className="schedule-threshold">Storage threshold: {schedule.threshold_percent}% remaining endurance</span></div></div>
    <div className="panel case-create-panel"><div className="panel-header"><div><h3>Create technician case</h3><p>Do not store passwords, recovery keys or other secrets in case notes.</p></div><span className="tiny-tag">SQLite</span></div><form className="case-form" onSubmit={submit}><label>Case title<input value={title} onChange={(event) => setTitle(event.target.value)} placeholder="e.g. Laptop overheating investigation" maxLength={120} required/></label><label>Device / client label<input value={device} onChange={(event) => setDevice(event.target.value)} placeholder="e.g. Dell Latitude 5420" maxLength={160}/></label><label>Initial notes<textarea value={notes} onChange={(event) => setNotes(event.target.value)} placeholder="Symptoms, user observations and first checks…" maxLength={8000}/></label><button className="primary-button" type="submit"><Save size={14}/> Save local case</button></form></div>
    <div className="panel"><div className="panel-header"><div><h3>Technician case history</h3><p>{cases.length} local case(s) · maximum 500 displayed</p></div><span className="tiny-tag">Local database</span></div>{cases.length ? <div className="case-list">{cases.map((item) => <article className="case-card" key={item.id}><div className="case-card-header"><div><strong>#{item.id} · {item.title}</strong><span>{item.device || "Device not specified"} · Updated {dateLabel(item.updated_at)}</span></div><button className="icon-button danger-icon" title="Delete case" onClick={() => onDelete(item.id)}><Trash2 size={14}/></button></div><textarea value={selectedNotes[item.id] ?? item.notes} onChange={(event) => setSelectedNotes((current) => ({ ...current, [item.id]: event.target.value }))} maxLength={8000}/><div className="case-card-footer"><select value={selectedStatus[item.id] ?? item.status} onChange={(event) => setSelectedStatus((current) => ({ ...current, [item.id]: event.target.value }))}><option value="open">Open</option><option value="in_progress">In progress</option><option value="resolved">Resolved</option><option value="archived">Archived</option></select><button className="subtle-button" onClick={() => onUpdate(item.id, selectedStatus[item.id] ?? item.status, selectedNotes[item.id] ?? item.notes)}><Save size={13}/> Save changes</button></div></article>)}</div> : <div className="history-empty">No cases yet. Create a case to begin tracking a repair or diagnostic session.</div>}</div>
  </div>;
}

function ReportsPage({ snapshot, storageScan, storageHistory, hardwareDiagnostics, updateStatus, faultReport, performanceResult, cases }: { snapshot: SystemSnapshot | null; storageScan: StorageScan | null; storageHistory: StorageScan[]; hardwareDiagnostics: HardwareDiagnostics | null; updateStatus: UpdateStatus | null; faultReport: FaultReport | null; performanceResult: PerformanceTestResult | null; cases: TechnicianCase[] }) {
  const report = {
    application: "Arcange CrossDiag",
    report_generated_at: new Date().toISOString(),
    system_inventory: snapshot,
    latest_storage_scan: storageScan,
    hardware_diagnostics: hardwareDiagnostics,
    update_status: updateStatus,
    fault_report: faultReport,
    latest_performance_test: performanceResult,
    technician_cases_count: cases.length,
    storage_scan_history_count: storageHistory.length,
    scope_note: "Collected evidence only. This report is not a guarantee of hardware health or remaining component lifespan.",
  };
  const reportText = JSON.stringify(report, null, 2);
  const [copied, setCopied] = useState(false);
  const downloadReport = () => { const blob = new Blob([reportText], { type: "application/json" }); const url = URL.createObjectURL(blob); const anchor = document.createElement("a"); anchor.href = url; anchor.download = `crossdiag-report-${new Date().toISOString().replaceAll(":", "-")}.json`; anchor.click(); URL.revokeObjectURL(url); };
  const printPdf = () => window.print();
  return <div className="stack-layout"><div className="panel report-card"><div className="report-file-icon"><FileText size={26}/></div><div className="report-copy"><h3>Diagnostic evidence export</h3><p>Export the current system inventory and latest diagnostic evidence as JSON, or create a formatted PDF using the system print dialog.</p><div className="report-meta"><span><CheckCircle2 size={13}/> {snapshot ? "System inventory available" : "No system snapshot"}</span><span><ShieldCheck size={13}/> Processed locally</span><span><HardDrive size={13}/> {storageHistory.length} storage scan(s) saved</span><span><ClipboardList size={13}/> {cases.length} technician case(s)</span></div></div><div className="report-actions"><button className="subtle-button" onClick={printPdf}><Printer size={14}/> Save as PDF</button><button className="primary-button" onClick={downloadReport}><Download size={15}/> Export JSON</button></div></div><div className="panel"><div className="panel-header"><div><h3>Report preview</h3><p>Latest collected evidence</p></div><button className="subtle-button" onClick={() => { void navigator.clipboard?.writeText(reportText).then(() => setCopied(true)).catch(() => setCopied(false)); }}>{copied ? "Copied" : "Copy JSON"}</button></div><pre className="json-preview">{reportText}</pre></div><article className="print-report"><h1>Arcange CrossDiag — Diagnostic Report</h1><p>Generated: {new Date(report.report_generated_at).toLocaleString()}</p><h2>Computer</h2><p>{snapshot ? `${snapshot.manufacturer} ${snapshot.model} · ${snapshot.os_name} ${snapshot.os_version} · ${snapshot.cpu_name}` : "No system inventory collected."}</p><p>Hostname: {snapshot?.hostname ?? "Not reported"} · RAM: {snapshot ? formatBytes(snapshot.total_memory_bytes) : "Not reported"}</p><h2>Storage</h2>{storageScan?.disks.length ? storageScan.disks.map((disk) => <p key={disk.device}><strong>{disk.model ?? disk.device}</strong> — {disk.status} — {disk.remaining_endurance_percent === null ? "Endurance unavailable" : `${disk.remaining_endurance_percent.toFixed(1)}% estimated remaining endurance`} — {disk.status_reason}</p>) : <p>No storage SMART scan is available.</p>}<h2>Hardware findings</h2>{hardwareDiagnostics ? <><p>Problem devices: {hardwareDiagnostics.problem_devices.length}; sensors reported: {hardwareDiagnostics.sensors.length}; drivers listed: {hardwareDiagnostics.drivers.length}.</p>{hardwareDiagnostics.limitations.map((item) => <p key={item}>{item}</p>)}</> : <p>Hardware diagnostics have not been collected.</p>}<h2>System events</h2>{faultReport?.faults.length ? faultReport.faults.slice(0, 30).map((fault, index) => <p key={`${fault.source}-${index}`}>{fault.severity} · {fault.source} · {fault.code ?? "No code"}: {fault.message}</p>) : <p>No fault report is available.</p>}<h2>Performance test</h2><p>{performanceResult ? `${performanceResult.test_id}: ${performanceResult.status} — ${performanceResult.summary}` : "No performance smoke test has been run."}</p><h2>Scope and limitations</h2><p>{report.scope_note}</p><p>Unsupported or missing values are not treated as passing tests. Physical inspection and dedicated test equipment may be required.</p></article></div>;
}

function SettingsPage({ snapshot, readiness, loading, error, onCheck }: { snapshot: SystemSnapshot | null; readiness: RuntimeReadiness | null; loading: boolean; error: string | null; onCheck: () => void }) {
  const availableCount = readiness?.tools.filter((tool) => tool.available).length ?? 0;
  const totalCount = readiness?.tools.length ?? 0;
  return <div className="stack-layout">
    <div className="panel about-card"><div className="about-mark"><MonitorCog size={31}/></div><div><div className="eyebrow">OPEN-SOURCE PROJECT</div><h2>Arcange CrossDiag</h2><p>Cross-platform PC diagnostics and technician workflow. Created as a successor direction to Arcange Windows Technician Toolkit.</p><div className="about-tags"><span>Windows + Linux</span><span>Rust + Tauri</span><span>React + TypeScript</span><span>MIT License</span><span>v0.10.0</span></div></div></div>
    <ApplicationUpdatePanel />
    <section className="panel runtime-panel">
      <div className="panel-header"><div><h3>Runtime readiness</h3><p>Check which optional diagnostic tools are available on this computer.</p></div><button className="subtle-button" onClick={onCheck} disabled={loading}>{loading ? <LoaderCircle className="spin" size={15}/> : <RefreshCw size={15}/>} {loading ? "Checking…" : "Check again"}</button></div>
      {error && <div className="inline-error" role="alert">{error}</div>}
      {loading && !readiness ? <div className="loading-state"><LoaderCircle className="spin" size={22}/><strong>Checking available tools…</strong><span>This only checks whether known tools are present on PATH.</span></div> : readiness ? <>
        <div className="runtime-summary"><div><span className="eyebrow">CURRENT ENVIRONMENT</span><strong>{readiness.platform} · {readiness.architecture}</strong><span>Checked {dateLabel(readiness.checked_at)}</span></div><div className="runtime-count"><strong>{availableCount}/{totalCount}</strong><span>tools detected</span></div></div>
        <div className="runtime-tools">{readiness.tools.map((tool) => <div className="runtime-tool" key={tool.id}><div className={`runtime-tool-status ${tool.available ? "available" : "missing"}`}>{tool.available ? <CheckCircle2 size={16}/> : <AlertTriangle size={16}/>}</div><div className="runtime-tool-copy"><strong>{tool.label}</strong><span>{tool.category} · {tool.purpose}</span>{!tool.available && <small>{tool.install_hint}</small>}</div><span className={`tiny-tag ${tool.available ? "" : "muted"}`}>{tool.available ? "Available" : "Not found"}</span></div>)}</div>
        <div className="runtime-limitations"><strong>Important limitations</strong><ul>{readiness.limitations.map((item) => <li key={item}>{item}</li>)}</ul></div>
      </> : <div className="empty-state"><div className="empty-icon"><CircleHelp size={22}/></div><strong>Readiness not checked</strong><p>Run the check to see which diagnostic utilities can be discovered.</p></div>}
    </section>
    <div className="detail-grid"><DetailCard icon={ShieldCheck} title="Privacy defaults" rows={[["Cloud telemetry", "Not included"], ["Remote control", "Not included"], ["Automatic repairs", "Confirmation required"], ["Data collection", "User initiated"]]}/><DetailCard icon={Settings2} title="Runtime details" rows={[["App version", snapshot?.app_version], ["OS", snapshot?.os_name], ["Last inventory", dateLabel(snapshot?.collected_at)]]}/></div>
    <div className="notice-card"><BookOpenText size={18}/><div><strong>Contribution principle</strong><p>Keep platform-specific functionality in typed diagnostic adapters. Every check must document its data source, required permissions, false-positive risks and a way to report “unsupported” rather than faking a result.</p><a className="external-link" href="https://github.com/arcange9/arcange-windows-technician-toolkit" target="_blank" rel="noreferrer">Open existing toolkit repository <ExternalLink size={13}/></a></div></div>
  </div>;
}
function LoadingState({ label }: { label: string }) { return <div className="loading-state"><LoaderCircle className="spin" size={24}/><strong>{label}</strong><span>Collecting facts only. No repairs are running.</span></div>; }
function EmptyState({ icon: Icon, title, description }: { icon: LucideIcon; title: string; description: string }) { return <div className="empty-state"><div className="empty-icon"><Icon size={22}/></div><strong>{title}</strong><p>{description}</p></div>; }
