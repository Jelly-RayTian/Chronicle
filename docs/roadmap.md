# Roadmap

## Milestone 0: Foundation

Installable Tauri shell, strict bilingual UI, SQLite migrations, typed status/folder/timeline boundaries, and automated quality gates.

## Milestone 1: Folder management and metadata snapshots

Explicit folder consent, cancellable metadata-only traversal, atomic complete snapshots, progress/warnings/recovery, and no timeline events.

## Milestone 2: Snapshot comparison and real timeline

- Atomic created, modified, deleted, and unchanged reconciliation.
- Immutable event history retained after observed deletion.
- Complete-scan-only deletion safety and startup recovery.
- SQLite-backed filtering, search, keyset pagination, file history, and scan history.
- Today, Yesterday, This week, and Earlier UI groups.
- Present-file open/reveal with deleted-file disabled states.

Milestone 2 does not add watching, rename/move detection, hashing, version families, projects, sessions, content search, AI, analytics, or destructive file operations.

## Milestone 3: Real-time watching and event coalescing

- Explicit per-folder native filesystem monitoring, disabled by default.
- Enable, disable, pause, resume, status, and error controls.
- Authorized-root validation, metadata-only recheck, debounce, classification, coalescing, and transactional watcher event publication.
- Manual/startup reconciliation for missed watcher events.

Milestone 3 does not add confirmed rename/move detection, full hashing, version families, projects,
sessions, content search, AI, hidden startup monitoring, analytics, or destructive file operations.

## Milestone 4: File identity, rename/move detection, and path history

- Platform-specific `identity_key` (Unix device/inode, Windows metadata fingerprint).
- Atomic rename/move detection during complete scans.
- Confirmed and heuristic event types with user-reviewable confidence.
- `file_path_history` for old_path → new_path transitions.

Milestone 4 does not add automatic confirmation, full hashing, version families, projects, sessions,
content search, AI, analytics, or destructive file operations.

## Milestone 5: Version-family suggestions and manual confirmation

- Heuristic grouping of related file versions by name-stem, version-token overlap, folder proximity,
  and stable identity keys.
- Explicit `suggested` / `confirmed` / `rejected` / `superseded` family states with `pending` / `accepted`
  / `rejected` decisions stored in SQLite.
- User actions: accept, reject, split, merge, rename family/members, add member, remove member.
- Versions page with confidence scores, evidence notes, and approximate chronological ordering.

Milestone 5 does not generate suggestions automatically, modify original files, infer version order
with certainty, add projects/sessions, content search, AI, analytics, or destructive file operations.

## Later exploration

Any identity inference, content indexing, or higher-level grouping requires a separate milestone, privacy review, and tests.
