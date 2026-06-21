# Roadmap

## Milestone 0: Foundation

- Installable Tauri application
- Strict bilingual React interface
- SQLite and versioned migrations
- Typed status, folder-list, and timeline commands
- Automated frontend and Rust quality gates

## Proposed Milestone 1

- Native folder selection with explicit user consent
- One-time metadata-only scan
- Scan progress, cancellation, warnings, and recovery
- Real file records and created events from the initial scan

Milestone 1 should not add watching, rename detection, project grouping, full-text search, AI, startup monitoring, or destructive file operations. It is proposed only and is not implemented automatically.

## Later exploration

Filesystem watching, file history, rename and move confidence, version families, project grouping, activity sessions, and optional local full-text search require separate design and privacy review.
