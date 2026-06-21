# Chronicle Agent Guide

## Product boundary

Chronicle is a local, privacy-first desktop application. Do not turn it into a static site, cloud drive, employee-monitoring product, analytics dashboard, or AI wrapper.

## Architecture rules

- React owns presentation, navigation, localization, and async UI states.
- Only `src/lib/tauri/client.ts` may call Tauri `invoke`.
- Rust owns database, filesystem, platform, event, and task logic.
- Tauri command wrappers must remain thin and delegate to testable functions.
- Use typed errors. Do not expose database paths or raw SQL errors to the UI.
- Use versioned SQLite migrations. Never mutate production schema ad hoc.

## Scope rules

- Do not create fake folders, events, charts, or productivity metrics.
- Do not add non-functional production buttons.
- Do not read file contents without a future milestone explicitly authorizing it.
- Do not implement scanning, watching, rename detection, grouping, AI, or destructive file operations in Milestone 0.

## Quality rules

- Use test-driven development for behavior changes.
- Keep TypeScript strict and free of `any`.
- Avoid `unwrap` and `expect` in recoverable Rust production paths.
- Keep Chinese and English resource keys identical.
- Use UTF-8 for Markdown, JSON, TOML, YAML, Rust, and TypeScript.
- Run all frontend and Rust checks before declaring completion.
- Run the Tauri application and visually inspect both languages and all states.

## Visual rules

- Keep the interface calm, original, professional, and desktop-first.
- Do not copy Finder, Explorer, Notion, or another product.
- Use one accent family, consistent radii, semantic tokens, Phosphor icons, and restrained motion.
- Verify light, dark, narrow-window, loading, empty, and error states.
