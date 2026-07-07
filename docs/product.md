# Product

Chronicle helps people return to local files through the time and context around their work. It is a privacy-first desktop history of meaningful metadata changes inside folders the user explicitly authorizes.

## Product boundaries

Chronicle is not surveillance software, a productivity score, cloud drive, backup system, destructive organizer, content search engine, or AI assistant. Original files remain under the user's control.

## Milestone 3 acceptance

A completed metadata scan reconciles its full discovery set against the last complete snapshot and atomically records created, modified, and deleted events. Unchanged rescans create no duplicate events. Any failed, cancelled, interrupted, or partial scan keeps the prior snapshot and cannot generate deletion events.

The timeline is backed by SQLite and supports stable pagination plus filename, extension, event type, folder, date, and current present/deleted filters. It groups events into Today, Yesterday, This week, and Earlier; shows file details, per-file event history, and folder scan history; and clearly disables open/reveal for deleted files.

Explicitly enabled folder monitoring records best-effort metadata changes through native filesystem watching. Monitoring is disabled by default, can be paused/resumed/disabled, validates every raw event against the authorized root, and uses reconciliation scans for recovery. Chronicle does not claim watcher history is complete and does not infer confirmed renames or moves.
