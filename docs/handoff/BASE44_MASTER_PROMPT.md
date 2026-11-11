# MASTER PROMPT — Build Arcange CrossDiag Cloud in Base44

You are a senior product architect, Base44 full-stack engineer, security engineer and UI/UX designer. Build **Arcange CrossDiag Cloud**, a professional cross-platform PC technician fleet-management and diagnostics portal that complements the existing Arcange CrossDiag Windows/Linux desktop application.

## Critical architectural rule

The supplied source project is a native **Rust + Tauri 2 + React + TypeScript desktop application**. It performs local hardware and operating-system diagnostics. Do NOT pretend a Base44-hosted browser app can directly access a user's SMART devices, BIOS, Windows Event Log, Linux journal, GPU sensors, USB ports, fans, drivers or OS package managers. Do NOT replace real hardware measurements with fake/demo values. Do NOT execute shell, PowerShell, firmware, driver or disk commands in Base44 cloud functions.

Build a **web companion and cloud backend** in Base44. Keep the native Rust/Tauri agent responsible for local device access. The desktop agent will eventually send user-approved, minimized diagnostic snapshots to Base44 over authenticated HTTPS. Base44 stores, manages and visualizes that data.

Use the Base44-supported frontend approach selected for this app (React is preferred), Base44 entities/database, authentication, server-side TypeScript functions on Deno, the Base44 SDK and realtime features where appropriate. Do not invent unsupported backend frameworks or use a separate database unless the project requirements and Base44 account entitlements require it.

## Product objective

Create a reliable, accessible, responsive technician portal for individuals, school IT labs, repair shops and small IT support teams to manage multiple Windows and Linux PCs, view diagnostic evidence over time, prioritize problems, manage repair cases and generate reports.

Brand: **Arcange CrossDiag Cloud**
Tagline: **Understand. Diagnose. Maintain.**
Visual direction: professional technical dashboard; dark navy/charcoal base, cyan/blue accents, neutral grey surfaces, crisp status colors used sparingly, high contrast, accessible typography, clear tables, useful charts and polished responsive mobile/tablet/desktop layouts. Provide light/dark themes. Avoid a generic template look.

## Build phases — do not skip ahead

### Phase 1: App shell and authentication

Create a polished app shell with:
- Landing/sign-in page.
- Authenticated workspace layout.
- Left navigation: Overview, Computers, Scan History, Findings, Storage Health, Update Inventory, Technician Cases, Reports, Schedules, Vendor Library, Team Members, Audit Log, Settings.
- Top bar with workspace switcher, search, notification center, profile and theme toggle.
- Responsive collapsible navigation.
- Empty, loading, error, offline and permission-denied states.
- Demo data must be visibly labelled DEMO and never mixed with real connected devices.

Implement secure authentication with user profile, workspace/team membership and roles. Do not create a public admin password or hard-coded credentials.

### Phase 2: Data entities and access control

Implement the entities and fields described in `BASE44_DATA_MODEL.md`:
- Computer
- DeviceEnrollment
- ScanSession
- DiagnosticFinding
- StorageSnapshot
- UpdateInventory
- TechnicianCase
- DiagnosticSchedule
- AuditEvent

Enforce row-level access on every entity. A user must never see another user's devices or scan records. Team membership and roles must be checked server-side. Credential hashes and enrollment secrets must never be returned to the frontend.

### Phase 3: Dashboard and device management

Overview page:
- Total enrolled computers.
- Online/offline count based on last heartbeat and a documented timeout.
- Devices requiring attention.
- Critical/high findings.
- Recent scan activity.
- Storage health trend summary.
- Update inventory summary.
- Open/overdue technician cases.

Computers page:
- Search/filter by name, OS, manufacturer, model, status and last seen.
- Device table and card view.
- Enroll device action with short-lived one-time code.
- Revoke/unlink device action with confirmation.
- No serial number or MAC address required.

Computer detail page tabs:
- Overview: identity, OS, agent version, last seen and current status.
- Hardware: CPU, RAM, GPU, motherboard, battery, network and peripheral inventory.
- Storage: devices, SMART evidence, endurance trends and critical warnings.
- Sensors: temperature/fan measurements with the source shown.
- Findings: issue timeline, severity, evidence, confidence and recommended next step.
- Updates: discovered OS/application/driver/firmware update inventory. Show status only; do not remotely install.
- Scans: previous scans and comparisons.
- Cases: support cases linked to the device.

### Phase 4: Diagnostics presentation

Use the following status vocabulary consistently:
- healthy
- attention
- failed
- unknown
- not_tested

Use severities: info, warning, high, critical.

Every diagnostic finding must show:
1. What was detected.
2. Source/evidence and collection time.
3. Why it matters in plain language.
4. Confidence/limitations.
5. Recommended next step.

Never label an unknown measurement as healthy or failed. Never calculate a single overall “hardware life” percentage from unrelated components. SSD remaining endurance is distinct from NVMe Percentage Used (endurance consumed). Preserve the local app's configured threshold as a policy, default 90%, but describe it as a user policy—not a manufacturer's universal failure threshold.

Charts:
- Storage remaining endurance when real measurements are available.
- Temperature history with sensor source/unit.
- Finding counts over time.
- Scan frequency and update inventory history.
- Do not extrapolate lifespan unless the data and model are defensible; label forecasts as estimates.

### Phase 5: Technician cases and reports

Technician case functions:
- Create, assign, prioritize, comment, update status and resolve cases.
- Link computers, scan sessions and findings.
- Maintain an immutable/audited history of sensitive changes.
- Do not allow credentials, passwords or BitLocker recovery keys in notes; show a warning.

Reports:
- Device health summary.
- Scan comparison.
- Storage health report.
- Open findings and repair case report.
- Export permitted data to CSV/JSON and a printable report view.
- Enforce the same record permissions on exports as on normal pages.

### Phase 6: Secure desktop-agent enrollment and ingestion

Create server-side TypeScript functions (Deno runtime) for:
- `createEnrollmentCode`: authenticated user creates a random, one-time, short-lived code for one workspace.
- `exchangeEnrollmentCode`: local agent submits code over HTTPS; validate expiry, single use, user/workspace ownership, and rate limits; issue a revocable device-scoped credential. Store only a secure hash of the credential where feasible.
- `ingestDiagnosticSnapshot`: authenticate the device credential; validate schema version, payload size, timestamp, device status, allowed fields, numeric bounds and rate limits; resolve ownership on the server; store scan session, findings, storage snapshots and update inventory transactionally/with idempotency.
- `deviceHeartbeat`: update last-seen status using a device-scoped credential.
- `revokeDevice`: authenticated owner/team admin revokes device enrollment and credential.
- `exportDeviceReport`: authenticated export with ownership checks and an audit record.

Use idempotency/request IDs to avoid duplicate scans on retries. Reject expired credentials, revoked credentials, oversized bodies, unexpected types, invalid numeric ranges and stale/future timestamps outside an explicit tolerance. Add rate limits and structured audit events. Do not log bearer tokens or secret values.

The local agent is the only component that reads hardware. It sends outbound HTTPS requests to Base44; the web frontend must not call localhost to execute diagnostics. Add a consent screen in the desktop agent that shows exactly what categories and fields will be uploaded. Default to no cloud sync until user opts in.

### Phase 7: Security rules and abuse resistance

- No arbitrary remote command execution.
- No remote PowerShell/shell commands.
- No firmware flashing, BIOS setting changes, disk formatting, partition editing, bootloader changes, OS installation or unattended driver/update installation from the cloud portal.
- No private API keys in React/browser code.
- No public read access to computer inventory, findings, cases or audit records.
- No trusting owner IDs supplied by the client; derive ownership from authenticated user/device enrollment on the server.
- Validate all user input and uploaded payloads.
- Add retention/deletion/export controls and privacy notice.
- Use least privilege for user, technician, team-admin and platform-admin roles.

### Phase 8: Quality and delivery

- Provide tests for authentication, owner isolation, team isolation, enrollment one-time use/expiry/revocation, payload validation, idempotency, rate limits and export permissions.
- Build with reusable typed components and clear folder structure.
- Document all entities, functions, roles, privacy and agent contract.
- Use demo fixtures only in demo mode and label them.
- Add loading skeletons, error boundaries and accessible labels.
- Do not claim an integration works until tested with a real locally running CrossDiag agent.

## Deliverables

1. Working Base44 web application shell.
2. Entities and row-level permissions.
3. Responsive Overview, Computers, Computer Details, Findings, Storage, Cases, Reports, Schedules, Vendor Library, Team and Settings pages.
4. Secure device enrollment and snapshot ingestion functions.
5. Typed JSON schema for local agent telemetry.
6. Demo data isolated from production data.
7. Automated test plan and security checklist.
8. Setup documentation explaining how the existing Rust/Tauri desktop agent will integrate.

## Existing source to respect

The accompanying source tree is the current Rust/Tauri desktop project. Inspect its README, `src/lib/desktop.ts`, `src-tauri/src/lib.rs`, `src/components/StorageTrendPanel.tsx`, `src/components/ApplicationUpdatePanel.tsx`, `docs/architecture.md`, `docs/platform-coverage.md`, `docs/safe-operations.md`, `docs/vendor-support.md` and `docs/first-pc-test.md` before designing matching data fields. Do not discard or overwrite the desktop source. Reuse its terminology, status model and diagnostic limitations.

Start with Phase 1 and Phase 2. Show the proposed entity schemas and access rules before building the ingestion functions. Then proceed phase by phase and report what is actually implemented and tested.
