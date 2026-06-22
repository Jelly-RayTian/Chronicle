# Chronicle

Chronicle is a privacy-first desktop application for rediscovering local files through time and context. It is being built as a real Tauri application, not a website, cloud drive, employee-monitoring tool, analytics dashboard, or AI chat wrapper.

## Milestone 1

The current milestone adds consent-based folder indexing and metadata snapshots to the production foundation:

- a Tauri 2 desktop shell with React and strict TypeScript;
- a Rust service core behind typed Tauri commands;
- a local SQLite database with versioned migrations;
- Timeline, Indexed folders, and Settings views;
- native folder selection, persistent indexed roots, availability states, and explicit index removal;
- cancellable, batched Rust metadata scans that keep the last complete snapshot;
- real loading, empty, progress, warning, cancellation, and error states;
- Simplified Chinese and English interfaces;
- automated frontend and Rust tests.

Milestone 1 does **not** watch the filesystem, read file contents, hash files, infer moves or renames, generate timeline events, group projects, or perform destructive file operations.

## Privacy guarantees

- Chronicle has no analytics or telemetry.
- Chronicle has no cloud storage and does not upload paths or metadata.
- Scanning is limited to folders the user explicitly selects.
- Metadata scanning never opens file contents and skips symbolic links by default.
- Failed, cancelled, or interrupted scans preserve the last complete snapshot.
- Clearing Chronicle data will never delete original files.

See [Privacy](docs/privacy.md) and [Security](SECURITY.md).

## Prerequisites

- Windows 11 with WebView2 Runtime
- Node.js 24 and npm 11
- Rust stable 1.96 or newer with the `x86_64-pc-windows-msvc` target
- Visual Studio 2022 Build Tools with Desktop development with C++ and a Windows SDK

## Development

```powershell
npm install
npm run tauri dev
```

Frontend checks:

```powershell
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

Rust checks:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Build the Windows installer:

```powershell
npm run tauri build
```

## Architecture at a glance

React renders the interface and calls one typed client. Tauri commands validate the boundary and delegate to Rust services. Rust owns SQLite, migrations, platform contracts, task contracts, and future filesystem work. React components never contain SQL or filesystem logic.

Read [Architecture](docs/architecture.md) and [Database](docs/database.md) for details.

## Documentation

- [Product](docs/product.md)
- [Architecture](docs/architecture.md)
- [Database](docs/database.md)
- [Event model](docs/event-model.md)
- [Privacy](docs/privacy.md)
- [Platform limitations](docs/platform-limitations.md)
- [Roadmap](docs/roadmap.md)
- [Testing](docs/testing.md)

## Project status

Chronicle is at Milestone 1. The database and UI are intentionally empty on first launch because no fake folders, files, or events are created.
