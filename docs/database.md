# Database

Chronicle uses SQLite through Rust `rusqlite`. SQLite is bundled to reduce Windows environment differences. Migrations are embedded in the binary and tracked with SQLite `user_version`.

## Conventions

- Primary keys use `INTEGER PRIMARY KEY` and map to Rust `i64`.
- Timestamps use UTC RFC 3339 text.
- Boolean values use checked integers.
- Foreign keys are enabled for every connection.
- Paths are stored normalized, but platform-specific comparison rules remain in the platform boundary.

## Initial schema

`indexed_folders` records explicitly approved roots, display name, scan time, monitoring preference, and availability.

`files` records metadata and presence under one indexed folder. `(indexed_folder_id, normalized_path)` is unique.

`file_events` records created, modified, deleted, renamed, or moved events with detection time, optional filesystem time and paths, confidence, and source.

`scan_runs` records scan lifecycle and non-negative counters.

`app_settings` stores native key-value settings in future milestones. Milestone 0 creates no fake settings rows.

## Indexes

Timeline queries use descending event time and id indexes. Folder-scoped event queries and scan-run history have dedicated indexes. Timeline pagination currently uses descending event id as a stable keyset cursor.

## Data clearing

Deleting Chronicle's database only clears Chronicle data. It must never delete original files. No destructive original-file operation exists in Milestone 0.
