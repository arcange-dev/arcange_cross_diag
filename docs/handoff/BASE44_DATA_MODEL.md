# Proposed Base44 data model

These are implementation specifications for Base44 entities. Adapt the exact JSON schema syntax to the current Base44 CLI/editor and verify the generated permission syntax in the app's security panel before use.

## Entities

### 1. `Computer`
- `owner_user_id` (required string)
- `team_id` (optional string)
- `display_name` (required string, max 100)
- `device_public_id` (required unique random ID; not a serial number)
- `manufacturer` (optional string)
- `model` (optional string)
- `platform` (enum: `windows`, `linux`, `unknown`)
- `os_name`, `os_version`, `architecture` (optional strings)
- `agent_version` (optional string)
- `status` (enum: `online`, `offline`, `attention`, `unlinked`)
- `last_seen_at` (optional datetime)
- `enrolled_at` (datetime)
- `revoked_at` (optional datetime)

### 2. `DeviceEnrollment`
- `owner_user_id`, `computer_id`
- `enrollment_code_hash` (never store plaintext)
- `expires_at`, `used_at`, `revoked_at`
- `credential_hash` (never return to frontend)
- `created_at`

### 3. `ScanSession`
- `owner_user_id`, `computer_id`
- `schema_version`, `agent_version`, `platform`
- `scan_type` (enum: `quick`, `full`, `storage`, `hardware`, `updates`, `manual`)
- `started_at`, `completed_at`
- `overall_status` (enum: `healthy`, `attention`, `failed`, `unknown`, `not_tested`)
- `summary` (bounded string)
- `limitations` (bounded array of strings)

### 4. `DiagnosticFinding`
- `owner_user_id`, `computer_id`, `scan_session_id`
- `category` (enum: `storage`, `cpu`, `memory`, `gpu`, `thermal`, `battery`, `network`, `driver`, `os`, `firmware`, `security`, `other`)
- `code`, `title`, `message`
- `severity` (enum: `info`, `warning`, `high`, `critical`)
- `status` (enum: `healthy`, `attention`, `failed`, `unknown`, `not_tested`)
- `source`, `confidence` (bounded string/number)
- `recommendations` (bounded string array)
- `created_at`, `resolved_at`

### 5. `StorageSnapshot`
- `owner_user_id`, `computer_id`, `scan_session_id`
- `device_path` (avoid serial numbers)
- `model`, `bus_type`, `capacity_bytes`
- `smart_status`, `critical_warning`
- `endurance_remaining_percent` (nullable numeric 0–100)
- `endurance_source` (nullable string)
- `temperature_c` (nullable numeric)
- `media_errors` (nullable integer)
- `available_spare_percent` (nullable numeric)
- `power_on_hours` (nullable integer)
- `collected_at`

### 6. `UpdateInventory`
- `owner_user_id`, `computer_id`, `scan_session_id`
- `package_id`, `display_name`, `current_version`, `available_version`
- `source` (package manager/provider)
- `update_type` (enum: `os`, `driver`, `application`, `firmware`, `other`)
- `reboot_required` (nullable boolean)
- `status` (enum: `available`, `installed`, `failed`, `unknown`)
- `collected_at`

### 7. `TechnicianCase`
- `owner_user_id`, `team_id`, `computer_id`
- `case_number`, `title`, `description`
- `status` (enum: `open`, `in_progress`, `waiting`, `resolved`, `closed`)
- `priority` (enum: `low`, `normal`, `high`, `critical`)
- `assignee_user_id` (optional)
- `finding_ids` (bounded array)
- `created_at`, `updated_at`, `resolved_at`

### 8. `DiagnosticSchedule`
- `owner_user_id`, `computer_id`
- `enabled`, `frequency` (enum: `daily`, `weekly`, `monthly`)
- `categories` (bounded array)
- `last_run_at`, `next_run_at`
- `created_at`

### 9. `AuditEvent`
- `actor_user_id`, `team_id`, `computer_id` (optional)
- `event_type`, `target_type`, `target_id`
- `summary` (bounded, no secrets)
- `created_at`

## Access-control policy

- A normal user can read only computers they own and the related records with the same `owner_user_id`.
- Team members can read only records associated with teams they belong to, according to an explicit team role.
- Technician role can create/update cases for assigned teams but cannot change ownership or device credentials.
- Team admin can manage team membership and devices within their team, but cannot access another team's data.
- Platform admin functions must be server-side only and audited.
- `DeviceEnrollment.credential_hash` and all credential-related fields must never be exposed to frontend reads.
- Client-side filters are not security controls; enforce ownership in Base44 row-level permissions and every server function.

## Data retention

Define retention by entity type. Example starting policy: telemetry snapshots 12 months, raw event messages 90 days, cases according to team policy, audit events 12 months. Allow user export/deletion where legally and operationally appropriate. Do not store raw command output unless a specific diagnostic workflow requires a bounded excerpt and the user consents.
