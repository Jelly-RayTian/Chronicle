# Chronicle portfolio overview

## The problem

Local work becomes hard to rediscover across folders and revisions, while many
history tools demand cloud upload or broad behavioral monitoring. Chronicle
builds a useful timeline from the smallest practical source: file metadata in
folders the user explicitly selects.

## Product

Chronicle is a Windows desktop application built with Tauri 2, React, strict
TypeScript, Rust, and bundled SQLite. It scans atomically, reconciles immutable
events, coalesces explicit watcher activity, searches filenames, optionally
indexes selected local text, and offers user-reviewable versions, projects, and
sessions. It never presents heuristic context as fact.

## Engineering depth

- Typed React → Tauri → Rust boundary; filesystem and SQL stay out of components.
- Ordered migrations, staged scan publication, rollback, keyset pagination, and retained history.
- Canonical roots, symlink rejection, bounded watchers/payloads/content reads, and sanitized diagnostics.
- Temporary filesystem/database tests, migration preservation, scale benchmarks, and Windows release automation.
- Bilingual UI, keyboard/focus accessibility, honest states, and real-product release presentation.

## Why v2.0.0 matters

Earlier milestones proved the application logic. v2.0.0 proves the project can
be responsibly handed to a stranger: what it does is visible, what it does not
do is explicit, the installer is reproducible and verifiable, failure and
uninstall paths preserve originals, and each public claim has a test, audit,
workflow, screenshot, or documented limitation behind it.
