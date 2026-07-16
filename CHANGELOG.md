# Changelog

All notable changes to Chronicle are documented here. The format follows Keep a Changelog, and the project uses semantic versioning once releases begin.

## [Unreleased]

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
