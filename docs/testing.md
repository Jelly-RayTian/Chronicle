# Testing

## Frontend

Vitest and React Testing Library cover typed command payloads, bilingual resource parity, navigation, loading/error/empty states, folder workflows, explicit monitoring controls, real timeline grouping, 300 ms debounced filename search, deleted-file details with disabled open/reveal actions, version-family list rendering, accept/reject/split/merge actions, member add/remove flow, approximate chronological ordering, project and session navigation, the explicit "no productivity score" guarantee, search-page navigation, mode toggles, and sanitized snippet display.

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
- missing indexed folder after restart preserving file records and rejecting traversal;
- missing roots and partial staged-count mismatch rollback;
- symbolic-link skipping and original-file non-mutation;
- SQLite keyset pagination plus filename, extension, event type, folder, date, and presence filters.
- watcher control-state persistence, authorized-root rejection, duplicate modify coalescing,
  create-then-modify and temporary-file patterns, folder removal while watching, startup
  reconciliation after missed events, event-storm error state, and watcher transaction rollback;
- version-family heuristic scoring/clustering (version-token sequences, unrelated same-extension files,
  cross-folder proximity, chronological ordering, stable identity-key boosts on Unix-style keys);
- version-family repository mutations (accept, reject, split, merge, add member, remove member,
  duplicate-member rejection, superseded-state handling);
- project creation, membership edits, suggestion grouping, accept/reject decisions, duplicate-member
  rejection, and project timeline loading;
- session generation (time-gap splitting, project linking, title editing) and explicit verification that
  session summaries contain no productivity scores.
- content-indexing disabled-by-default behavior, explicit per-folder opt-in, TXT/Markdown/source-code extraction,
  default and hard file-size limits, V10 database enforcement, default secret-file exclusions, incremental update after file changes, deletion sync,
  folder disable and clear-index removal, sanitized snippet escaping, and filename-only search.
- watcher raw-path deduplication before canonicalization, bounded storm handling, large-folder UI warning,
  and fixed native timeline/search page limits.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Tests never use a production Chronicle database or modify files outside temporary fixtures.

## Performance suite

The ignored release-profile suite generates temporary 1,000- and 10,000-file
fixtures by default and accepts an optional 50,000-file count. It measures scan
staging, atomic reconciliation after deterministic moves, warmed timeline and
filename-search medians, watcher burst coalescing, and SQLite bytes. It has no
absolute timing assertion because hardware and filesystem conditions vary.

See [Performance benchmarks](benchmarks.md) for the exact command, environment,
methodology, results, and limitations.

## Visual verification

Run `npm run tauri dev` with an isolated application-data profile and verify Timeline, Indexed folders, Settings, Versions, Projects, Sessions, and Search in English and Simplified Chinese. Inspect normal and narrow windows, light/dark themes, active filters, pagination, detail drawer, present/deleted actions, monitoring enable/pause/resume/disable states, version-family suggestions with confidence/evidence, accept/reject/split/merge/add/remove actions, project creation/suggestion/accept/reject/member editing, session generation/accept/reject/title editing, content-indexing enable/disable/reindex/edit controls, filename and content search modes, sanitized snippets, long paths, keyboard focus, and empty/error states. A visual claim requires a current screenshot or direct inspection. See [Accessibility](accessibility.md) and [Smoke test](smoke-test-checklist.md).

## Release validation

`npm run release:validate -- --tag v2.0.0` checks stable semver alignment
across npm, Cargo, Cargo.lock, and Tauri and requires the matching release-notes
file. After building, `./scripts/validate-windows-artifacts.ps1` verifies one
NSIS PE installer, records its Authenticode state, and writes
`dist/release/SHA256SUMS.txt`.

## Continuous integration

CI runs frontend formatting, lint, TypeScript, tests, production build, Rust formatting, Clippy with warnings denied, tests, and check on Windows.
