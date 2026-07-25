# Database

Chronicle uses bundled SQLite through Rust `rusqlite`. Migrations are embedded, ordered, and tracked by `PRAGMA user_version`.

## Schema through version 10

- `indexed_folders`: explicitly approved canonical roots and availability state.
- `files`: the latest complete metadata state. Deleted files remain as `is_present = 0` so history survives. Includes `identity_key` for stable file identification.
- `file_events`: immutable observations from reconciliation and watcher batches. Event types: created, modified, deleted, renamed, moved, likely_renamed, possible_move. Includes `user_confirmation` and `user_confirmed_at` for inferred event review.
- `scan_runs`: complete scan history including completed, failed, cancelled, and startup-recovered runs.
- `scan_file_staging`: discovery rows owned by one running scan. Includes `identity_key`.
- `watcher_states`: explicit monitoring intent, runtime status, coalescing window, last error, and watcher counters.
- `app_settings`: reserved native settings storage.
- `file_path_history`: tracks old_path → new_path transitions per file with valid_from, valid_until, confidence, evidence, and event source.
- `version_families`: user-reviewable version-family records with `status` (`suggested`, `confirmed`, `rejected`, `superseded`) and `decision` (`pending`, `accepted`, `rejected`).
- `version_family_members`: ordered members of a family, each pointing to one `files` row. A member stores a zero-based `version_index` and a label used in the UI.
- `version_family_suggestions`: candidate file pairs produced by the heuristic suggestion pass, with confidence, evidence notes, and whether the suggestion has been `handled`.
- `projects`: user-created or suggested project groups with `status` (`suggested`, `active`, `archived`, `rejected`) and an optional `decision`.
- `project_members`: files assigned to a project, either `manual` or `suggested`.
- `project_suggestions`: candidate file-to-project assignments from the suggestion pass, with confidence, evidence, source, and `handled` state.
- `activity_sessions`: inferred activity sessions with title, optional project link, start/end times, event summary, and `status` (`auto`, `edited`, `accepted`, `rejected`).
- `activity_session_events`: links a session to the `file_events` that formed it.
- `activity_session_files`: distinct files referenced by events in a session.

V3 adds indexes for filename/extension/presence filtering, event-type timeline queries, and per-file event history. Released V1 and V2 migrations are unchanged.
V4 adds watcher state for Milestone 3. Released V1 through V3 migrations are unchanged.
V5 adds file identity tracking, path history, rename/move detection, and user confirmation. The file_events table is recreated to support the new event types. Released V1 through V4 migrations are unchanged.
V6 adds version-family tables, suggestion tracking, and user decision states. Released V1 through V5 migrations are unchanged.
V7 adds project groups, project membership, project suggestions, activity sessions, session event links, and session file links. Released V1 through V6 migrations are unchanged.
V8 adds per-folder optional content-indexing configuration (`content_indexing_enabled`, `content_indexing_extensions`, `content_indexing_max_bytes`, `content_indexing_exclusion_patterns`), the `content_index_documents` tracking table, and the SQLite FTS5 virtual table `content_index_fts`. Released V1 through V7 migrations are unchanged.
V9 adds persisted event favorites. Released V1 through V8 migrations are unchanged.
V10 clamps legacy content-index size settings and adds insert/update triggers enforcing the 1-byte to 8-MiB hard range. Released V1 through V9 migrations are unchanged.

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

## Version family suggestions

The suggestion pass is invoked manually by the user and runs after the current `files` snapshot is already published. It groups present files into candidate version families using name-stem similarity, version-token overlap, folder proximity, and optional identity-key agreement. Each candidate pair becomes a `version_family_suggestions` row. Conflicting pairs are reconciled into `version_families` rows with `status = 'suggested'` and `decision = 'pending'`, and their members are written to `version_family_members` in chronological order.

The UI distinguishes suggested (`decision = 'pending'`) families from confirmed (`status = 'confirmed'`, `decision = 'accepted'`) and rejected/superseded records. Accepting a suggestion sets `status = 'confirmed'` and `decision = 'accepted'`. Rejecting sets `decision = 'rejected'`. Splitting a family marks the original as `superseded` and creates a new confirmed family. Merging families marks the source families as `superseded` and creates a new confirmed family. Removing the last member of a confirmed family sets its decision to `rejected`. All mutations commit in SQLite transactions so the family list and its members remain consistent.

## Project groups and activity sessions

Projects are explicit groups of file records. A project may be user-created (`status = 'active'`, members `manual`) or suggested (`status = 'suggested'`, members `suggested`). The suggestion pass considers folder proximity, filename keywords, temporal co-occurrence, confirmed version families, and Git repository membership. Each candidate assignment becomes a `project_suggestions` row. Accepting a suggested project marks it active and converts suggested members to manual; rejecting marks it rejected. Manual membership edits insert or delete `project_members` rows and mark the relevant suggestion as handled.

Activity sessions are generated from `file_events` on explicit user request. The generator sorts events by `detected_at`, splits on a configurable time gap, and caps session size. It assigns an optional `project_id` when session files overlap an active project. Sessions are inserted with `status = 'auto'`. User edits change the title and/or project and set `status = 'edited'`. Accept/reject set `accepted`/`rejected`. The generator replaces existing `auto` sessions on each run, leaving edited or reviewed sessions untouched.

## Optional local full-text indexing

Content indexing is disabled by default. Each `indexed_folders` row stores an explicit opt-in flag, allowed extensions, a per-file byte limit, and exclusion patterns. The default is 1 MiB and the V10 database plus Rust reader enforce an 8 MiB hard maximum. When enabled, Chronicle reads eligible present files (regular files, allowed extension, under the size limit, not matching an exclusion pattern) inside the authorized root, extracts text as lossy UTF-8, and stores it in the local SQLite FTS5 table `content_index_fts`.

The `content_index_documents` table tracks the last indexed size and timestamp for each file. A sync pass compares this state to the current `files` snapshot and reindexes only changed or missing files. Disabling a folder deletes its FTS rows and documents; clearing the index deletes all rows. Removing an indexed folder also clears its content-index rows before the folder row is deleted.

Search supports two modes:

- **Filename search**: SQLite `LIKE` on `files.name`, available regardless of content-indexing state.
- **Content search**: FTS5 `MATCH` on `content_index_fts`, joined to present `files` rows. Results include a snippet with custom highlight markers; the backend strips those markers and HTML-escapes the snippet before returning it to the UI.

No remote API, cloud upload, or AI inference is used. File contents are never logged.

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

Timeline requests are clamped to 100 rows. Filename/content search requests are
clamped to 200 rows. The production UI requests 30 timeline rows and 50 search
rows, preventing a single native call from returning an unbounded payload.
