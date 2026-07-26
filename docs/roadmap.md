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

## Milestone 6: Project groups and activity sessions

- User-created project groups that organize related records without moving files on disk.
- Suggested project groups based on folder proximity, filename keywords, repeated temporal co-occurrence,
  confirmed version families, Git repository membership, and user labels.
- Confidence, evidence, accept, reject, edit, and persisted decisions for projects.
- Activity sessions generated only from Chronicle file events using temporal proximity, project relation,
  folder relation, and event density.
- Session start/end, related files, event summary, optional project link, and editable title.
- Projects page, project details, project timeline, Sessions view, and session details.

Milestone 6 does not add full-text content search, AI, application/window/keyboard/mouse/browser/screenshot
tracking, automatic file organization, productivity scores, analytics, or destructive file operations.

## Milestone 7: Optional local full-text search

- Explicit per-folder opt-in for local content indexing, disabled by default.
- Supported formats: `.txt`, `.md`, and selected source-code text files, with configurable extensions.
- Per-folder file-size limits, exclusion patterns, and default secret-file exclusions (`.env`, `.env.*`, `*.key`, `*.pem`, `id_rsa`, etc.).
- Local text extraction and SQLite FTS5 index.
- Incremental update after scans and watcher batches, removal when disabled, and clear-content-index action.
- Filename-only versus content-search modes with clear UI labels.
- Sanitized snippets; arbitrary HTML from FTS5 is escaped and never rendered as trusted markup.
- No remote API, upload, OCR, PDF/DOCX extraction, or natural-language AI search.

Milestone 7 does not add automatic cloud indexing, AI search, binary-file indexing, OCR, PDF/DOCX extraction, or productivity scoring.

## v1.4.0: Performance & Scale

- Reproducible 1,000-, 10,000-, and optional 50,000-file temporary fixtures.
- Separate scan, 10%-move reconciliation, timeline, filename-search, watcher-burst, and SQLite-footprint measurements.
- Measured scan staging and watcher duplicate-path optimizations.
- Large-folder warning, 8 MiB content-index hard cap, bounded timeline/search payloads, and watcher storm stop.
- Browser rendering containment for off-screen timeline and search cards.

v1.4.0 does not add a speculative storage rewrite, new tracking source, analytics,
cloud services, productivity metrics, or unrelated features.

## v2.0.0: Public release

- Aligned release metadata and exact-tag validation.
- Windows NSIS build, PE/size/signature validation, and published SHA-256 digest.
- Full quality gates before GitHub Release creation.
- Real screenshots, release notes, smoke checklist, accessibility review, privacy/security audit, and portfolio overview.
- Visible keyboard focus, skip navigation, meaningful control names, and dialog keyboard behavior.

v2.0.0 adds no AI, cloud sync, telemetry, hidden monitoring, or major product
feature. The next product milestone remains intentionally undecided until
public-release feedback is reviewed.

## Later exploration

Any additional identity inference, higher-level grouping, or new extraction formats require a separate milestone, privacy review, and tests.
