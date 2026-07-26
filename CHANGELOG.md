# Changelog

All notable changes to Chronicle are documented here. The format follows Keep a Changelog, and the project uses semantic versioning once releases begin.

## [Unreleased]

## [2.0.0] - 2026-07-27

### Added

- Version/tag and Windows artifact validation, including SHA-256 generation and explicit Authenticode status reporting.
- Public release, smoke-test, accessibility, privacy/security audit, portfolio, demo GIF, and known-limitations documentation.
- Real application screenshots captured with an isolated profile and temporary authorized files.
- Skip-to-content navigation and keyboard focus containment/restoration for release-critical dialogs.

### Changed

- Aligned npm, Cargo, and Tauri manifests at v2.0.0.
- Made tag releases run every frontend and Rust quality gate before building and publishing.
- Updated public presentation and release documentation.

### Security

- Re-audited telemetry/network absence, explicit-root enforcement, opt-in content indexing, sanitized diagnostics, and original-file non-mutation.
- Release artifacts now fail validation when missing, malformed, unexpectedly small, or in an invalid signature state.

## [1.4.0] - 2026-07-25

### Added

- Reproducible release-profile benchmarks for 1,000, 10,000, and optional 50,000-file generated fixtures, covering scan, reconciliation, timeline, filename search, watcher bursts, and SQLite footprint.
- Large-folder warning after a scan observes at least 10,000 files.
- V10 migration enforcing a 1-byte to 8-MiB content-indexing size range.
- Honest benchmark methodology, results, limits, and v1.4.0 release notes.

### Changed

- Increased metadata staging batches from 256 to 1,024 rows and reduced frontend scan-status polling to 750 ms.
- Deduplicate repeated raw watcher paths before authorized-root canonicalization.
- Skip off-screen timeline and search-card layout and paint work using browser rendering containment.
- Bumped application manifests to v1.4.0.

### Security

- Content reads are capped at 8 MiB even when an older or malformed configuration requests more.
- Watcher burst optimization retains canonical authorized-root validation and the existing 4,096-unique-path storm stop.

## [1.2.0] - 2026-07-07

### Added

- Merge sessions command: combine two adjacent sessions into one. Moves all events and files from source to target, recomputes time range and event summary, and deletes the source session.
- Session card improvements: duration display, per-type event breakdown (created/modified/deleted tags), project chip.
- Session details now show event-type breakdowns and formatted timestamps with duration.
- Documented session inference algorithm: conservative time-gap splitting (default 30 min), project-majority assignment, 100 event cap per session, 1000 session cap.

## [1.1.0] - 2026-07-07

### Added

- Timeline grouping now includes This month and Older sections alongside Today, Yesterday, and This week.
- Quick time filters: Today, Yesterday, Last 7 days, Last 30 days buttons and a jump-to-date picker.
- Event favorites: star any timeline event and use a favorites filter to find them later. Favorites are persisted in a new `favorite_events` table (V9 migration).
- Keyboard shortcuts: `/` focuses search, `j`/`k` or arrow keys navigate events, `Enter` opens detail panel, `Escape` closes detail panel or clears filters.
- Confidence badge on event cards showing the inference percentage for inferred events.
- Copy path button in the detail panel to copy a file's full path.

### Changed

- Event cards now show a confidence badge when the event has sub-1.0 confidence from inference.
- Detail panel has a Copy path action and a star/unstar button for favorites.
- Active event gets a visible focus ring for keyboard navigation.

## [1.0.1] - 2026-07-07

### Changed

- Improved all user-facing error messages to consistently explain what happened, whether original files are safe, and what the user can do next.
- Added a specific error for database migration failures, separate from generic database-unavailable errors, with a message that includes recovery steps.
- Added a specific error for content-indexing-disabled attempts so users are told to enable content indexing rather than seeing a generic failure.
- Startup now logs watcher and scan initialization errors instead of silently ignoring them; migrated folders with watchers enabled still reconcile on restart.
- The database initialization phase now captures migration failure details and surfaces them through the typed error boundary.

### Added

- Test verifying that a missing indexed folder after restart is surfaced correctly, preserves file records from previous scans, and rejects traversal on the deleted path.
- Test verifying every new error code maps to the correct i18n message key and retryable flag.

### Fixed

- Cancelled scan, missing-folder, permission-denied, watcher-failure, event-storm, and file-unavailable error messages now include explicit file-safety reassurances and recovery guidance.
- Interrupted startup watchers/scan errors are no longer silently discarded.

## [1.0.0-rc.1] - 2026-07-07

### Added

- Tauri 2, React, TypeScript, Vite, and Rust project foundation.
- Local SQLite database with versioned migrations from schema V1 through V8.
- Typed commands for application info, database status, indexed folders, and timeline pagination.
- Bilingual Timeline, Indexed folders, Settings, Search, Versions, Projects, and Sessions views.
- Loading, empty, progress, warning, cancellation, and safe error states.
- Frontend and Rust quality checks with GitHub Actions, plus a Windows packaging job.
- Native indexed-folder selection, canonical registration, duplicate prevention, and nested-root warnings.
- Metadata-only Rust scanning with progress, cancellation, batching, symbolic-link skipping, and atomic snapshot publication.
- Persistent folder availability, scan-run recovery, and explicit Chronicle-index removal that leaves originals untouched.
- Transactional snapshot reconciliation for created, modified, deleted, and unchanged files.
- Immutable file-event history retained after observed deletion, with safe failed/cancelled/interrupted recovery.
- SQLite-backed timeline search, filters, keyset pagination, file details, event history, and scan history.
- Today, Yesterday, This week, and Earlier grouping with explicit deleted-file action states.
- Explicit per-folder native filesystem monitoring, disabled by default, with enable, disable, pause, resume, status, and error controls.
- Watcher event validation, debounce, metadata recheck, temporary-file filtering, coalescing, transactional publication, and startup reconciliation for explicitly enabled folders.
- Opt-in per-folder content indexing for `.txt`, `.md`, and supported source-code files, with local filename and content search.
- Suggested and user-editable version families for related file revisions.
- Suggested and user-editable projects with manual membership and project timelines.
- Activity sessions inferred from file event time gaps and project links.
- Sanitized diagnostics export in Settings that includes counts and status only—no paths, names, or contents.
- Migration upgrade test from V1 schema to the latest schema with data preservation.
- Scan performance tests for small, medium, and large temporary directory trees.
- Windows NSIS installer packaging and a GitHub Actions release workflow triggered by `v*.*.*` tags.
- Bug-report and feature-request issue templates.
