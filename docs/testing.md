# Testing

## Frontend

Vitest and React Testing Library verify application rendering, navigation, real empty states, understandable database failure, loading behavior, the typed Tauri client, and Chinese/English resources.

```powershell
npm test
```

Native calls are mocked only at the typed client boundary. Components do not fabricate folders or events.

## Rust

Rust tests use temporary or in-memory SQLite databases. They verify migration success, required tables, empty folder and timeline queries, command behavior, and camelCase model serialization.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Tests must never open a user's production Chronicle database.

## Visual verification

Launch `npm run tauri dev`. Inspect Timeline, Indexed folders, and Settings in Simplified Chinese and English. Verify light and dark themes, normal and narrow windows, keyboard focus, loading, empty, and database error states. Save screenshots for delivery evidence.

## Continuous integration

GitHub Actions runs formatting, lint, TypeScript, frontend tests, production frontend build, Rust formatting, clippy, tests, and check on Windows.
