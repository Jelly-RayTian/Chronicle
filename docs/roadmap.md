# Roadmap

## Milestone 0: Foundation

- Installable Tauri application
- Strict bilingual React interface
- SQLite and versioned migrations
- Typed status, folder-list, and timeline commands
- Automated frontend and Rust quality gates

## Milestone 1: Folder management and metadata snapshots

- Native folder selection with explicit user consent
- One-time metadata-only scan
- Scan progress, cancellation, warnings, and recovery
- Real file metadata records with atomic latest-snapshot publication
- No timeline events yet

Milestone 1 does not add watching, event generation, rename detection, project grouping, full-text search, AI, startup monitoring, or destructive file operations.

## Later exploration

Filesystem watching, file history, rename and move confidence, version families, project grouping, activity sessions, and optional local full-text search require separate design and privacy review.
