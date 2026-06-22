# Database

Chronicle uses SQLite through Rust `rusqlite`. SQLite is bundled to reduce Windows environment differences. Migrations are embedded in the binary and tracked with SQLite `user_version`.

## Conventions

- Primary keys use `INTEGER PRIMARY KEY` and map to Rust `i64`.
- Timestamps use UTC RFC 3339 text.
- Boolean values use checked integers.
- Foreign keys are enabled for every connection.
- Paths are stored normalized, but platform-specific comparison rules remain in the platform boundary.

## Schema through version 2

`indexed_folders` records explicitly approved canonical roots, a platform comparison key, display name, scan time, monitoring preference, availability reason, and last availability check.

`files` stores only the latest successful metadata snapshot, including parent path. `(indexed_folder_id, normalized_path)` is unique.

`file_events` records created, modified, deleted, renamed, or moved events with detection time, optional filesystem time and paths, confidence, and source.

`scan_runs` records scan lifecycle, non-negative counters, and a safe failure category. `scan_file_staging` contains discovery batches for one running scan.

`app_settings` stores native key-value settings in future milestones. Milestone 0 creates no fake settings rows.

## Indexes

Timeline queries use descending event time and id indexes. Folder-scoped event queries and scan-run history have dedicated indexes. Timeline pagination currently uses descending event id as a stable keyset cursor.

## Data clearing

Removing an indexed folder cascades only through Chronicle tables. No SQL value is used as a target for a filesystem delete, move, or rename operation.

## Atomic snapshot publication

Discovery batches commit only to `scan_file_staging`. After complete traversal, one transaction upserts observed metadata, removes rows absent from the new complete snapshot, updates the folder's successful-scan time, completes `scan_runs`, and clears staging. Failed, cancelled, and startup-recovered scans clear staging without changing `files`. `file_events` remains empty in Milestone 1.
