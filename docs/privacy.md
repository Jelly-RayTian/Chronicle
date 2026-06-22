# Privacy

Chronicle is local and consent-based by design.

## Guarantees

- No analytics, behavioral tracking, or telemetry.
- No cloud storage or metadata upload.
- Indexing is limited to canonical roots the user explicitly selects.
- Scans read directory entries and filesystem metadata only; they do not open file contents.
- Symbolic links are skipped by default, and scan commands resolve roots from the authorization database rather than accepting paths.
- Chronicle does not monitor folders automatically at startup.
- Clearing Chronicle data never deletes original files.
- Removing a folder index deletes only Chronicle database rows.

## Stored metadata

Chronicle stores the file name, canonical path, parent path, extension, byte size, available creation time, modification time, first-indexed time, and last-seen time. This metadata remains local and can still be sensitive; the application does not log or upload it.

## Data location

The SQLite database is stored in the operating system application-data directory resolved by Tauri. The UI does not receive the absolute database path.

## Future full-text search

Any future content indexing must be optional, local, clearly disclosed, and separately consented. It is outside Milestone 0.
