# Chronicle

Chronicle is a privacy-first desktop application for rediscovering local files through time and context. It is being built as a real Tauri application, not a website, cloud drive, employee-monitoring tool, analytics dashboard, or AI chat wrapper.

## Milestone 0

The current milestone provides the production foundation:

- a Tauri 2 desktop shell with React and strict TypeScript;
- a Rust service core behind typed Tauri commands;
- a local SQLite database with versioned migrations;
- Timeline, Indexed folders, and Settings views;
- real loading, empty, and error states;
- Simplified Chinese and English interfaces;
- automated frontend and Rust tests.

Milestone 0 does **not** select folders, scan the filesystem, watch changes, read file contents, generate file events, group projects, or perform destructive file operations.

## Privacy guarantees

- Chronicle has no analytics or telemetry.
- Chronicle has no cloud storage and does not upload paths or metadata.
- Future scanning will be limited to folders the user explicitly selects.
- Milestone 0 never reads file contents.
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

Chronicle is at Milestone 0. The database and UI are intentionally empty on first launch because no fake folders or events are created.
