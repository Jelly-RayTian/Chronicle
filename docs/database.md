# Database

Chronicle uses bundled SQLite through Rust `rusqlite`. Migrations are embedded, ordered, and tracked by `PRAGMA user_version`.

## Schema through version 5

- `indexed_folders`: explicitly approved canonical roots and availability state.
- `files`: the latest complete metadata state. Deleted files remain as `is_present = 0` so history survives. Includes `identity_key` for stable file identification.
- `file_events`: immutable observations from reconciliation and watcher batches. Event types: created, modified, deleted, renamed, moved, likely_renamed, possible_move. Includes `user_confirmation` and `user_confirmed_at` for inferred event review.
- `scan_runs`: complete scan history including completed, failed, cancelled, and startup-recovered runs.
- `scan_file_staging`: discovery rows owned by one running scan. Includes `identity_key`.
- `watcher_states`: explicit monitoring intent, runtime status, coalescing window, last error, and watcher counters.
- `app_settings`: reserved native settings storage.
- `file_path_history`: tracks old_path → new_path transitions per file with valid_from, valid_until, confidence, evidence, and event source.

V3 adds indexes for filename/extension/presence filtering, event-type timeline queries, and per-file event history. Released V1 and V2 migrations are unchanged.
V4 adds watcher state for Milestone 3. Released V1 through V3 migrations are unchanged.
V5 adds file identity tracking, path history, rename/move detection, and user confirmation. The file_events table is recreated to support the new event types. Released V1 through V4 migrations are unchanged.

## Exact reconciliation algorithm

For a completed discovery set `C` and the last published snapshot `P`, scoped to one indexed folder and normalized path:

- Created: `path ∈ C` and no row in `P` is currently present. This includes a formerly deleted path that reappears (but not a renamed/moved file whose identity matched).
- Modified: `path ∈ C ∩ P`, the prior row is present, and at least one relevant field differs: name, parent path, extension, byte size, available creation time, or modification time.
- Deleted: the prior row is present and `path ∈ P \ C`, and no rename/move match was found.
- Renamed/Moved: a deleted path's identity_key matched a created path's identity_key (confirmed). If the parent directory is the same it is `renamed`; otherwise `moved`.
- Likely renamed / possible move: heuristic match by same size + close modification time + filename similarity when identity_key is unavailable or differs.
- Unchanged: all relevant fields match. No event is inserted; only `last_seen_at` advances.
- Ambiguous: if no identity or heuristic match exceeds threshold, preserve separate delete and create events.

## Publication transaction

`complete_scan` opens one transaction and:

1. verifies the scan id belongs to the folder and is still `running`;
2. verifies staged row count equals the traversal's files-seen count;
3. computes all created, modified, and deleted differences before mutating `files`;
4. detects rename/move pairs by identity_key matching and heuristic similarity;
5. for matched pairs: updates the existing file row path and inserts rename/move event + path history;
6. upserts remaining staged current files while preserving `first_indexed_at`;
7. marks prior-but-absent rows `is_present = 0` instead of deleting them;
8. inserts immutable reconciliation events referencing retained file ids;
9. updates the folder's successful-scan and availability fields;
10. marks the scan completed with final counters;
11. clears that scan's staging rows;
12. commits.

Any error rolls back every step. The scanner then records a failed run and clears staging in a separate cleanup transaction. It cannot leave a half-published snapshot or events without the corresponding snapshot.

## Watcher transaction

Each debounced watcher batch is scoped to one indexed folder and commits in one transaction:

1. verify the folder still exists;
2. compare each final metadata observation to the current `files` row;
3. upsert present files (including identity_key) or mark previously present missing paths as absent;
4. insert immutable `file_events` with `event_source = 'watcher'`;
5. increment watcher counters and `last_event_at`;
6. commit.

If a batch fails, the transaction rolls back and no partial watcher event is published. Watcher events use lower confidence than complete reconciliation because native filesystem notifications can be duplicated, delayed, dropped, or coalesced by the operating system.

## Failure and startup recovery

A cancellation request first acquires the database boundary and atomically changes a still-running scan to `cancelled` while clearing staging; publication and cancellation are therefore serialized, and only one can win. Failed and cancelled scans update only `scan_runs` and delete their staging rows. On startup, any abandoned `running` scan becomes failed and abandoned staging is removed. Neither path changes `files` or `file_events`. For folders whose monitoring was explicitly enabled, Chronicle runs a startup reconciliation scan and then restarts watching; this catches missed events without claiming watcher history is complete.

## Queries and retention

Timeline pagination is stable descending keyset pagination by event id. Filters execute in SQLite, not React. File event history, path history, and folder scan history have bounded limits. Removing an indexed folder intentionally clears only its Chronicle database rows through foreign-key cascades; original files are never a SQL deletion target.
