# Privacy

Chronicle is local and consent-based by design.

## Guarantees

- No analytics, behavioral tracking, or telemetry.
- No cloud storage or metadata upload.
- Future indexing is limited to folders the user explicitly selects.
- Milestone 0 does not read file contents.
- Chronicle does not monitor folders automatically at startup.
- Clearing Chronicle data never deletes original files.

## Data location

The SQLite database is stored in the operating system application-data directory resolved by Tauri. The UI does not receive the absolute database path.

## Future full-text search

Any future content indexing must be optional, local, clearly disclosed, and separately consented. It is outside Milestone 0.
