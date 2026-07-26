# Product

Chronicle helps people return to local files through the time and context around their work. It is a privacy-first desktop history of meaningful metadata changes inside folders the user explicitly authorizes.

## v2.0.0 acceptance

Public release preparation does not expand what Chronicle observes. It makes
the existing product independently understandable and releasable: versions and
tags align, Windows artifacts are validated, release notes and limitations are
complete, keyboard and focus behavior are reviewed, and privacy/data-safety
claims have code and test evidence.

## v1.4.0 acceptance

Performance claims must be reproducible with generated temporary fixtures and
must name the machine, build profile, fixture shape, measurement boundary, and
known limitations. Large-folder behavior retains the same authorization,
atomic-publication, watcher-storm, and original-file safety rules as smaller
folders. This release improves measured bottlenecks only; it does not use scale
work as permission to add new tracking or product features.

## Product boundaries

Chronicle is not surveillance software, a productivity score, cloud drive, backup system, destructive organizer, content search engine, or AI assistant. Original files remain under the user's control.

## Milestone 3 acceptance

A completed metadata scan reconciles its full discovery set against the last complete snapshot and atomically records created, modified, and deleted events. Unchanged rescans create no duplicate events. Any failed, cancelled, interrupted, or partial scan keeps the prior snapshot and cannot generate deletion events.

The timeline is backed by SQLite and supports stable pagination plus filename, extension, event type, folder, date, and current present/deleted filters. It groups events into Today, Yesterday, This week, and Earlier; shows file details, per-file event history, and folder scan history; and clearly disables open/reveal for deleted files.

Explicitly enabled folder monitoring records best-effort metadata changes through native filesystem watching. Monitoring is disabled by default, can be paused/resumed/disabled, validates every raw event against the authorized root, and uses reconciliation scans for recovery. Chronicle does not claim watcher history is complete and does not infer confirmed renames or moves.
