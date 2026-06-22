# Testing

## Frontend

Vitest and React Testing Library verify application rendering, navigation, real empty states, folder selection/registration, nested warnings, explicit removal confirmation, understandable failures, typed commands, and synchronized Chinese/English resources.

```powershell
npm test
```

Native calls are mocked only at the typed client boundary. Components do not fabricate folders or events.

## Rust

Rust tests use temporary SQLite databases and directories. They verify migration and restart recovery, registration persistence, duplicates, nesting, traversal rejection, missing roots, metadata collection, empty and nested folders, symbolic-link skipping, cancellation and failed-scan safety, disappearing paths without events, and index removal without touching originals.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Tests must never open a user's production Chronicle database.

## Visual verification

Launch `npm run tauri dev`. Inspect Timeline, Indexed folders, and Settings in Simplified Chinese and English. Verify native folder selection, registration, scan progress/cancellation, explicit removal confirmation, light and dark themes, normal and narrow windows, keyboard focus, empty and error states. Save screenshots for delivery evidence.

## Continuous integration

GitHub Actions runs formatting, lint, TypeScript, frontend tests, production frontend build, Rust formatting, clippy, tests, and check on Windows.
