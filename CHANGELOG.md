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
