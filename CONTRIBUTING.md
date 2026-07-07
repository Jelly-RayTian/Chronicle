# Contributing to Chronicle

Thank you for helping build Chronicle carefully.

## Setup

Install the prerequisites listed in the README, then run:

```powershell
npm install
npm run tauri dev
```

## Development workflow

1. Create a focused branch or worktree.
2. Write a failing test for new behavior.
3. Implement the smallest change that passes the test.
4. Keep React, Tauri command, service, and persistence boundaries intact.
5. Run frontend, Rust, and visual checks.
6. Update documentation when behavior or architecture changes.

## Required checks

```powershell
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Before a release, also verify the migration upgrade test:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migration_from_v1_preserves_indexed_folder_and_files -- --nocapture
```

## Pull requests

Describe the user-facing result, privacy impact, tests added, and screenshots for visual changes. Do not include generated data, local database files, credentials, or private paths.

Use clear commits such as `feat: add timeline pagination` or `fix: preserve folder consent state`.

## Releases

Releases are built from tags matching `v*.*.*`. Pushing such a tag triggers the release workflow, which builds the Windows installer and attaches it to a GitHub Release.
