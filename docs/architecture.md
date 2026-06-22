# Architecture

## Layers

```text
React UI
  -> typed Tauri client
    -> thin Tauri commands
      -> Rust services
        -> SQLite persistence
        -> task, scanner, event, and platform contracts
```

## Frontend

`src/app` owns startup state and navigation. `src/pages` renders product views. `src/components` contains layout and reusable states. `src/i18n` contains matching Chinese and English resources. `src/lib/tauri/client.ts` is the only native invocation boundary.

## Native core

`src-tauri/src/commands` converts Tauri state into service calls. `folders` owns registration and removal rules. `scanner` performs metadata-only traversal, `tasks` owns in-memory cancellation tokens, `platform` owns normalization and availability checks, and `database` owns migrations, staging, and snapshot publication. `events` remains unused in Milestone 1.

## Startup data flow

Tauri resolves the platform application-data directory, opens `chronicle.sqlite3`, applies embedded migrations, marks interrupted scans failed, clears abandoned staging rows, and manages the database plus scan-task registry as application state. React requests application info, database status, indexed folders, and the empty timeline independently.

## Milestone 1 scan flow

The native dialog returns a user-selected directory. Rust rejects traversal components, symbolic-link roots, non-directories, and exact duplicates, then stores the canonical root. A scan command accepts only the folder id and re-reads the authorized path from SQLite. A blocking worker walks regular files without opening contents or following symbolic links, writes metadata in 256-record staging batches, and exposes persisted progress for UI polling. One final transaction publishes the complete snapshot. Failure or cancellation deletes staging only.

## Boundary invariants

- React contains no SQL or filesystem operations.
- Commands contain no business rules beyond input and output boundary handling.
- Core services can be tested without a Tauri window.
- Absolute database paths and raw errors never cross into React.
- Scan commands never accept an arbitrary root path.
- No Milestone 1 code inserts `file_events`.
