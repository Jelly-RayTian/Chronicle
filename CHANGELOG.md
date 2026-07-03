# Changelog

All notable changes to Chronicle are documented here. The format follows Keep a Changelog, and the project uses semantic versioning once releases begin.

## [Unreleased]

### Added

- Tauri 2, React, TypeScript, Vite, and Rust project foundation.
- Local SQLite database with a versioned initial migration.
- Typed commands for application info, database status, indexed folders, and timeline pagination.
- Bilingual Timeline, Indexed folders, and Settings views.
- Loading, empty, and safe error states.
- Frontend and Rust quality checks with GitHub Actions.
- Native indexed-folder selection, canonical registration, duplicate prevention, and nested-root warnings.
- Metadata-only Rust scanning with progress, cancellation, batching, symbolic-link skipping, and atomic snapshot publication.
- Persistent folder availability, scan-run recovery, and explicit Chronicle-index removal that leaves originals untouched.
- Transactional snapshot reconciliation for created, modified, deleted, and unchanged files.
- Immutable file-event history retained after observed deletion, with safe failed/cancelled/interrupted recovery.
- SQLite-backed timeline search, filters, keyset pagination, file details, event history, and scan history.
- Today, Yesterday, This week, and Earlier grouping with explicit deleted-file action states.
- Explicit per-folder native filesystem monitoring, disabled by default, with enable, disable, pause, resume, status, and error controls.
- Watcher event validation, debounce, metadata recheck, temporary-file filtering, coalescing, transactional publication, and startup reconciliation for explicitly enabled folders.
