# Privacy

Chronicle is local, consent-based, and metadata-only.

## Guarantees

- Only canonical roots explicitly selected by the user can be scanned.
- Native monitoring is disabled by default and can only be enabled per explicitly selected folder.
- Scan commands use a stored folder id rather than accepting arbitrary paths.
- Scans inspect directory entries and filesystem metadata; file contents are not opened or indexed.
- Watcher batches recheck filesystem metadata only; file contents are not opened or indexed.
- Symbolic links are not followed.
- No analytics, telemetry, cloud storage, metadata upload, behavior monitoring, or productivity scoring exists.
- Cancelled, failed, interrupted, or partial scans preserve the last complete snapshot.
- Chronicle never deletes, moves, renames, or writes original files.
- A deletion event is only a historical observation after a complete scan; it is not a filesystem action.
- Watcher history is best-effort and Chronicle does not claim that it captures every filesystem event.

## Stored local data

Chronicle stores authorized root paths, filename/path/parent/extension, byte size, available creation and modification timestamps, first-indexed/last-seen times, presence, immutable file events, scan history, and explicit watcher status/error counters. This metadata can be sensitive and stays in the operating-system application-data SQLite database.

Open and reveal are explicit user actions. The command receives only a database file id, reloads the stored path and root, and rejects deleted, escaped, symbolic-link, missing, or non-file targets before invoking the platform. Chronicle never opens anything automatically.
