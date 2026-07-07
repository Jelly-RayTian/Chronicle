# Privacy

Chronicle is local, consent-based, and metadata-only.

## Guarantees

- Only canonical roots explicitly selected by the user can be scanned.
- Native monitoring is disabled by default and can only be enabled per explicitly selected folder.
- Scan commands use a stored folder id rather than accepting arbitrary paths.
- Scans inspect directory entries and filesystem metadata; file contents are not opened or indexed unless content indexing is explicitly enabled for a folder.
- Watcher batches recheck filesystem metadata only; file contents are not opened or indexed.
- Content indexing is opt-in per folder, restricted by extension and file size, and respects exclusion patterns. Indexed text is stored only in the local SQLite database.
- Symbolic links are not followed.
- No analytics, telemetry, cloud storage, metadata upload, behavior monitoring, or productivity scoring exists.
- Cancelled, failed, interrupted, or partial scans preserve the last complete snapshot.
- Chronicle never deletes, moves, renames, or writes original files.
- A deletion event is only a historical observation after a complete scan; it is not a filesystem action.
- Watcher history is best-effort and Chronicle does not claim that it captures every filesystem event.
- Diagnostics exports contain counts and status only; no file paths, names, or contents are included.

## Stored local data

Chronicle stores authorized root paths, filename/path/parent/extension, byte size, available creation and modification timestamps, first-indexed/last-seen times, presence, immutable file events, scan history, watcher status/error counters, and (only when enabled) indexed text contents. This data stays in the operating-system application-data SQLite database.

Open and reveal are explicit user actions. The command receives only a database file id, reloads the stored path and root, and rejects deleted, escaped, symbolic-link, missing, or non-file targets before invoking the platform. Chronicle never opens anything automatically.
