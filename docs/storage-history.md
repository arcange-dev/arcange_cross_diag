# Storage history persistence

## Database

CrossDiag stores storage scan history in the local `crossdiag.sqlite3` database under the application data directory. The `storage_scans` table contains a monotonically increasing row ID, scan timestamp and the serialized scan record. Writes are transactionally inserted and the table is trimmed to the newest 100 scans in the same transaction.

## Legacy migration

Older builds stored scans in `storage-history.json`. On first access, CrossDiag imports at most the newest 100 records into SQLite using an immediate transaction. The `app_settings.storage_history_json_migrated` marker is written only after the import succeeds. If the JSON is malformed, the migration fails without deleting or modifying the original file and without setting the marker. The original JSON remains as a backup after a successful migration. To bound startup memory and parsing work, files larger than 16 MiB are rejected without modification; the user can retain the backup and review it manually.

## Concurrency and durability

The SQLite connection uses WAL journaling, `synchronous=NORMAL`, and a five-second busy timeout. A manual scan and scheduled scan can safely append concurrently without read-modify-write history loss. A busy/locked database still reports a visible save limitation rather than discarding the current scan result.

## Privacy

The history records SMART fields already collected by the storage adapter. Device serial numbers are not added by the migration or new persistence layer. The database is local and is not uploaded automatically. Users should treat the app-data directory and exported reports as potentially sensitive technician records.
