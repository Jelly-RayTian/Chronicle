# Testing

## Frontend

Vitest and React Testing Library cover typed command payloads, bilingual resource parity, navigation, loading/error/empty states, folder workflows, explicit monitoring controls, real timeline grouping, 300 ms debounced filename search, deleted-file details with disabled open/reveal actions, version-family list rendering, accept/reject/split/merge actions, member add/remove flow, and approximate chronological ordering.

```powershell
npm test
```

Native behavior is mocked only at the typed Tauri client boundary. Timeline data is never fabricated in production components.

## Rust

Rust tests use temporary directories and SQLite databases. Coverage includes:

- first scan created events and an unchanged second scan with no duplicates;
- created, modified, deleted, and multiple simultaneous changes;
- successful deletion with retained file rows and history;
- cancelled and failed scans preserving the last complete snapshot;
- startup recovery of interrupted scans;
- missing roots and partial staged-count mismatch rollback;
- symbolic-link skipping and original-file non-mutation;
- SQLite keyset pagination plus filename, extension, event type, folder, date, and presence filters.
- watcher control-state persistence, authorized-root rejection, duplicate modify coalescing,
  create-then-modify and temporary-file patterns, folder removal while watching, startup
  reconciliation after missed events, event-storm error state, and watcher transaction rollback;
- version-family heuristic scoring/clustering (version-token sequences, unrelated same-extension files,
  cross-folder proximity, chronological ordering, stable identity-key boosts on Unix-style keys);
- version-family repository mutations (accept, reject, split, merge, add member, remove member,
  duplicate-member rejection, superseded-state handling).

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Tests never use a production Chronicle database or modify files outside temporary fixtures.

## Visual verification

Run `npm run tauri dev` and verify Timeline, Indexed folders, Settings, and Versions in English and Simplified Chinese. Inspect normal and narrow windows, light/dark themes, active filters, pagination, detail drawer, present/deleted actions, monitoring enable/pause/resume/disable states, version-family suggestions with confidence/evidence, accept/reject/split/merge/add/remove actions, long paths, keyboard focus, and empty/error states. A visual claim requires a current screenshot or direct inspection.

## Continuous integration

CI runs frontend formatting, lint, TypeScript, tests, production build, Rust formatting, Clippy with warnings denied, tests, and check on Windows.
