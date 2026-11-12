# Base44 architecture and migration plan

## 1. Recommended product split

### Local CrossDiag Agent (existing source)

Keep the existing Rust + Tauri 2 desktop application. It is responsible for:

- Reading local OS and hardware inventory.
- Running SMART/NVMe, sensor, CPU/RAM/GPU, driver, firmware and event-log checks where supported.
- Enforcing local permissions and operation allow-lists.
- Running user-approved local maintenance operations.
- Persisting local scan history and producing offline reports.
- Displaying measurements and limitations when offline.

It must not expose arbitrary PowerShell, shell or process execution to a cloud service.

### CrossDiag Cloud (new Base44 app)

Use Base44 for:

- Authentication and technician/team accounts.
- Computer/device enrollment and fleet inventory.
- Diagnostic snapshot history and trend views.
- Alerts, technician cases, assignments and notes.
- Reports and evidence attachments.
- Update inventory visibility (not remote unattended installation).
- Vendor support knowledge and model-specific documentation links.
- Team-level dashboards and audit history.

Base44's developer platform describes frontend flexibility, managed entities/database, authentication, server-side TypeScript functions on Deno, realtime, storage and hosting. Base44 also documents GitHub integration for version control and sync. Verify plan requirements and current product availability in your account before relying on specific features.

## 2. Data flow

1. User installs and opens the local CrossDiag Agent.
2. User signs into CrossDiag Cloud in a browser (or links a cloud account from the desktop agent).
3. Cloud creates a short-lived one-time enrollment code.
4. User explicitly approves linking the current computer.
5. The local agent exchanges the code over HTTPS for a device-scoped credential.
6. The local agent collects diagnostics locally and shows exactly which categories will be uploaded.
7. After explicit consent, the agent sends a minimized JSON snapshot to a Base44 server function over HTTPS.
8. The server function authenticates the device, validates the schema and limits, maps it to the authenticated owner, and stores records.
9. The cloud dashboard shows received data, timestamp, source, limitations and stale/offline status.

Use outbound HTTPS from the local agent. Do not make the cloud browser call `localhost` to run hardware commands. Avoid a local HTTP server unless a later design proves it necessary and secures origin checks, authentication, binding and CSRF protections.

## 3. Security requirements

- Require authentication for every dashboard route and server function except a deliberately public landing page.
- Enforce row-level/owner/team access on every entity; never rely on hiding buttons in the UI.
- Device enrollment codes must be one-time, random, short-lived, rate-limited and stored as hashes.
- Device credentials must be revocable and scoped to one device. Rotate credentials on re-enrollment.
- Store secrets only in Base44 server-side secrets/environment variables, never in frontend code or repository files.
- Use TLS for all device-to-cloud traffic.
- Validate payload size, schema, timestamps, device ownership, allowed metric names, string lengths and numeric ranges.
- Reject unknown fields or preserve them only in a bounded, explicitly versioned extension object.
- Do not upload serial numbers, MAC addresses, usernames, file paths, Wi-Fi details, BitLocker keys, passwords or arbitrary command output by default.
- Provide a user-visible upload preview and per-category consent.
- Add audit events for enrollment, revocation, exports, role changes and privileged administration.
- Apply rate limits, retention limits, deletion/export flows and abuse monitoring.
- No remote shell, remote PowerShell, arbitrary command strings, firmware flashing, disk formatting, partition editing, bootloader modification or unattended driver/update installation from Base44.

## 4. Snapshot contract

Every record should carry:

- `schema_version` (e.g. `1.0`).
- `device_id` (cloud identifier, not hardware serial).
- `agent_version` and `platform`.
- `collected_at` in ISO-8601 UTC.
- `category` and `source`.
- `status`: `healthy`, `attention`, `failed`, `unknown`, or `not_tested`.
- Measurement values with units and source.
- `limitations` and confidence/evidence metadata.

Never infer a healthy device from missing values. Preserve `unknown` as a distinct state. Keep the user's configurable SSD remaining-endurance threshold as a policy value; NVMe Percentage Used is endurance consumed, not remaining.

## 5. Suggested implementation phases

### Phase A — Base44 web shell
- Branding, authentication, responsive navigation and theme.
- Dashboard with explicit empty states and sample data clearly marked as demo.
- Device list/detail pages and technician case UI.

### Phase B — entities and access control
- Create entities from `BASE44_DATA_MODEL.md`.
- Add owner/team/admin access policies and tests.
- Add audit and retention rules.

### Phase C — telemetry ingestion
- Implement enrollment, exchange, snapshot ingestion, heartbeat and device revocation functions.
- Add payload validation and device-scoped authentication.
- Use fixtures generated from the local CrossDiag result types.

### Phase D — trend and case features
- Storage health trend plots, diagnostic finding timelines, update inventory, cases, assignments, reports and export.

### Phase E — agent integration
- Add a narrowly scoped outbound HTTPS sync client to the local Rust agent.
- User reviews and approves each category before upload.
- Handle offline queues, retries, duplicate request IDs and credential revocation.

### Phase F — production testing
- Test owner isolation, role escalation, invalid credentials, revoked devices, oversized payloads, replayed requests, missing sensor values and stale timestamps.
- Verify Base44 usage, rate limits, storage and plan constraints.

## 6. What not to migrate into Base44

Do not attempt to run `smartctl`, `nvidia-smi`, `Get-WinEvent`, `journalctl`, `Get-PnpDevice`, package managers, driver tools or firmware tools from Base44's cloud functions. They must run on the user's local computer through the Rust/Tauri agent. Base44 cloud functions may validate, store and analyze the structured diagnostic evidence received from that agent.
