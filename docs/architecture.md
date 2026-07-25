# Architecture

## Layers

```text
React UI
  -> typed Tauri client
    -> thin Tauri commands
      -> Rust services
        -> SQLite persistence
        -> scanner, event, task, and platform boundaries
```

React owns presentation state, debounced filters, timeline grouping, pagination controls, version-family review UI, and details panels. It does not enumerate files, construct SQL, infer events, compute version-family similarity, or invoke operating-system processes directly. `src/lib/tauri/client.ts` is the only native invocation boundary.

Rust resolves every scan from an indexed-folder id stored in SQLite. The scanner reads directory entries and metadata inside that authorized root without following symbolic links. Database code owns staging, reconciliation, immutable event persistence, filters, histories, and transaction boundaries. Platform code validates a present path against its authorized root before an explicit open or reveal request.

## v1.4 performance boundaries

Scanner discovery stages up to 1,024 metadata rows per SQLite transaction. This
is also the progress-persistence cadence; React polls running scan status every
750 ms. Timeline uses descending event-id keyset pagination, accepts at most 100
rows per native request, and the UI requests 30. Timeline and search cards use
browser rendering containment so off-screen cards do not require layout and
paint work.

Watcher bursts first discard identical raw paths, then canonicalize and validate
each unique path against the authorized root. No validation is skipped. A batch
may retain at most 4,096 unique pending paths; an overflow stops the watcher
without publishing synthetic deletion events.

## Milestone 2 scan and reconciliation flow

1. A thin command starts a scan by folder id.
2. Rust reloads the canonical authorized root from SQLite and creates a `running` scan row.
3. Traversal writes metadata to `scan_file_staging` in batches. The published `files` snapshot and `file_events` are untouched.
4. A missing entry, unreadable directory, cancellation, or other incomplete discovery fails the run. Staging is cleared and the prior complete snapshot remains authoritative.
5. After traversal, cancellation is checked again.
6. `complete_scan` performs reconciliation and publication in one SQLite transaction.
7. React polls scan status. Completion changes the timeline query generation, causing the database-backed page to refresh.

## Command boundaries

Timeline queries accept typed filename, extension, event type, folder, UTC date bounds, current-presence, cursor, and page-size values. File and scan histories are fetched by database identifiers. Open/reveal commands accept only a file id, reload its current path and authorized root from SQLite, reject deleted, missing, non-file, symbolic-link, and escaped paths, then pass the path as a process argument without shell interpolation.

## Invariants

- Commands are thin; reconciliation and SQL remain in Rust services.
- React contains no filesystem or database logic.
- A scan command never accepts an arbitrary root path.
- No incomplete scan can publish a snapshot or event. Cancellation and publication serialize on the same database connection, so a scan recorded as cancelled cannot publish.
- Created/modified/deleted events, file presence, folder status, and scan completion commit together.
- Rename/move detection runs inside the same transaction as publication. All path history and identity updates are atomic with the snapshot.
- Identity resolution uses platform-specific mechanisms (Unix inode/device, metadata fingerprinting on other platforms) without reading file contents.
- Heuristic renames are marked as likely_renamed or possible_move, never as confirmed, and are user-reviewable.
- No hashing, content search, or future-milestone behavior is present.
- Version-family suggestions are generated only on explicit user request; they never modify original files.
- A family member always references an existing `files` row; the repository rejects duplicate members within the same family.
- Suggested, confirmed, rejected, and superseded states are explicit in SQLite and reflected by the UI.
- Version ordering uses metadata timestamps and is approximate; the UI never claims exact version numbers from heuristics.
- Project groups organize file records without moving files on disk. Projects and sessions are metadata-only abstractions.
- Project suggestions are generated only on explicit user request and never modify original files or events.
- Activity sessions are inferred from Chronicle file events only; no keyboard, application, window, browser, mouse, or screenshot data is used.
- Session boundaries are approximate and use event timestamps; the UI never presents them as objective productivity metrics or scores.

## Milestone 3 watcher flow

Monitoring remains disabled by default for every indexed folder. When a user explicitly enables it,
Rust starts a native `notify` watcher for that folder only. React exposes controls and status, but it
does not receive raw filesystem paths or perform filesystem/database work.

Watcher processing is best-effort:

1. native raw event;
2. authorized-root validation for every event path;
3. normalized path key;
4. deterministic debounce/coalescing window;
5. metadata recheck with `symlink_metadata` without following symbolic links;
6. created/modified/deleted classification against the current `files` row;
7. one SQLite transaction for file-row update, immutable watcher event insert, and watcher status;
8. typed status polling causes the timeline query to refresh.

Duplicate modifies, rapid save sequences, create-then-modify bursts, common temporary files, and
atomic replacement patterns are reduced to the final meaningful metadata observation for a path.
If a watched folder becomes unavailable, Chronicle stops publishing watcher observations for that
folder and preserves the last complete snapshot. Event storms move the watcher into an error state
without manufacturing deletion events.

Startup recovery for explicitly enabled monitoring folders runs the existing complete metadata scan
before restarting watching. Manual reconciliation remains the normal "Scan metadata" action.
Watcher history is never described as perfectly complete.

## Milestone 5 version-family flow

1. The user opens the Versions page and clicks "Refresh suggestions".
2. A thin command loads present files from the current snapshot and runs the heuristic version-family service.
3. The service normalizes filenames, extracts version tokens, scores name similarity, version-token overlap, folder proximity, and identity-key agreement, and reconciles candidate pairs into family clusters.
4. Suggested families are written to `version_families` (`status = 'suggested'`, `decision = 'pending'`) with members ordered chronologically by modification time.
5. React displays the family list, confidence, evidence notes, and member ordering; users can accept, reject, split, merge, rename, add, or remove members.
6. Each action is a thin command backed by a repository transaction; success refreshes the family list in React.

Version families are explicitly user-reviewed. Suggestions do not modify files, events, or the `files` snapshot. Chronological ordering is approximate and uses metadata timestamps; it does not claim exact version numbers.

## Milestone 6 project groups and activity sessions

1. The user opens the Projects page and creates a project manually or clicks "Refresh suggestions".
2. A thin command loads present files, confirmed version families, recent file events, and indexed-folder roots.
3. The project suggestion service scores folder proximity, filename keywords, temporal co-occurrence, version-family membership, and Git repository membership.
4. Suggested projects are written to `projects` (`suggested`) with `project_members` (`suggested`) and `project_suggestions` rows.
5. The user can accept, reject, rename, add/remove members, or create projects manually; each action is a transactional repository mutation.
6. On the Sessions page, the user clicks "Generate sessions".
7. The session service sorts `file_events` by `detected_at`, splits by a gap threshold, assigns optional active projects, and writes `activity_sessions` + links.
8. Users can edit session titles/projects and accept/reject individual sessions.

Projects and sessions are purely contextual metadata. They never rename, move, delete, or read the contents of original files. Session inference uses only Chronicle file events; no external activity sources are consulted.

## Milestone 7 optional local full-text indexing

1. Content indexing is disabled by default for every indexed folder.
2. The user opts in per folder on the Indexed folders page, with visible defaults for extensions, max file size, and exclusion patterns.
3. After a successful metadata scan or watcher batch, the content-indexing service runs a sync pass for that folder if enabled.
4. The sync pass loads the folder config and current `files` snapshot, skips ineligible files, reads eligible text files inside the authorized root, and updates `content_index_documents` and `content_index_fts`.
5. The user searches from the Search page, choosing filename or content mode and optionally filtering by folder.
6. Content results return FTS5 snippets with custom markers; the backend strips markers and escapes HTML before serializing to the frontend. The frontend renders snippets as plain text.
7. Disabling a folder or clearing the index deletes the corresponding FTS rows and documents. Removing an indexed folder clears its content-index rows first.
