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

`src-tauri/src/commands` converts Tauri state into service calls. `database` owns connections, migrations, and repositories. `models` defines serialized DTOs. `errors` converts internal failures into safe application errors. `scanner`, `events`, `tasks`, and `platform` define future seams without performing filesystem work.

## Startup data flow

Tauri resolves the platform application-data directory, opens `chronicle.sqlite3`, applies embedded migrations, and manages the database as application state. React requests application info, database status, indexed folders, and the first timeline page independently. A failed request does not blank the whole window.

## Boundary invariants

- React contains no SQL or filesystem operations.
- Commands contain no business rules beyond input and output boundary handling.
- Core services can be tested without a Tauri window.
- Absolute database paths and raw errors never cross into React.
