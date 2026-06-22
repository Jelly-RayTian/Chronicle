# AGENTS.md

## Project

Chronicle is a production-oriented, privacy-first desktop application that indexes file metadata and activity only inside folders explicitly selected by the user. It is built with Tauri, React, TypeScript, Rust, and SQLite.

## Before changing code

1. Read relevant files under `docs/`.
2. Inspect current implementation and tests.
3. Restate the current task.
4. List files expected to change.
5. Identify filesystem, database, privacy, security, and data-loss risks.
6. Present a concise implementation plan.

## Engineering rules

- Implement only the current milestone.
- Do not begin future milestones automatically.
- Do not rewrite unrelated working code.
- Keep filesystem, database, analysis, and operating-system logic out of React components.
- Keep Rust core services separate from thin Tauri command wrappers.
- Use strict TypeScript and avoid `any`.
- Use typed Rust errors.
- Avoid `unwrap` and `expect` in recoverable production paths.
- Use ordered, versioned database migrations.
- Do not edit released migrations.
- Do not add dependencies without explaining why.
- Do not create fake production data or non-functional buttons.
- Treat paths, filenames, and project contents as untrusted input.
- Restrict native operations to explicitly authorized roots.
- Do not upload private data or add hidden analytics.
- Do not log file contents, source code, secrets, or tokens.
- Use temporary directories and temporary databases in tests.
- Never perform destructive operations against original user files.
- Do not track keyboard activity, browser history, active windows, screenshots, or mouse movement.
- Use neutral language and never create productivity scores.

## Required checks

Before declaring completion, run all applicable checks:

- frontend formatting
- frontend lint
- TypeScript type-check
- frontend tests
- frontend production build
- cargo fmt --check
- cargo clippy
- cargo test
- cargo check

Report failures and warnings honestly.

## Documentation

Update relevant documentation whenever architecture, database schema, privacy, security, or behavior changes. Never advertise unfinished features as complete.

## Teaching requirement

After each meaningful task, explain:

1. What changed.
2. End-to-end data flow.
3. React responsibilities.
4. Rust responsibilities.
5. SQLite responsibilities.
6. Important files and types.
7. Main algorithm.
8. One likely bug and debugging steps.
9. One small exercise for the repository owner.
10. Five interview questions with concise answers.
