# Event Model

File events are immutable metadata observations produced by complete scan reconciliation or, in Milestone 3, explicitly enabled filesystem watcher batches.

## Event types

- `created`: present in the new complete snapshot and not previously present (and not a confirmed rename/move target).
- `modified`: same normalized path, previously present, with relevant metadata differences.
- `deleted`: previously present and absent from the new complete snapshot (and not a confirmed rename/move source).
- `renamed`: confirmed rename (same directory, identity_key match). Confidence 1.0.
- `moved`: confirmed move across directories (identity_key match). Confidence 1.0.
- `likely_renamed`: heuristic rename inference (same size + close modification time + strong filename similarity). Confidence ~0.8.
- `possible_move`: heuristic move/rename inference (weaker signal). Confidence ~0.4.

Every event stores detection time, optional filesystem time, old/new path, confidence, source, folder id, file id, user confirmation status, and user confirmation timestamp.

## Milestone 4 identity and rename/move detection

File identity is resolved during scanning: on Unix, `(device, inode)` from `stat`; on other platforms, a metadata fingerprint (size + modification time + creation time). Matching priorities:

1. **Identity key match**: same stable ID → confirmed rename/move, confidence 1.0.
2. **Heuristic match**: same size + close modification time + filename similarity → likely_renamed (0.8) or possible_move (0.4).
3. **No match**: separate deleted + created events are preserved as before.

User can confirm or reject inferred events; confirmation is persisted in the `user_confirmation` column.

Path history records every old_path → new_path transition in `file_path_history` with valid_from, valid_until, confidence, evidence, and source.

## False-event prevention

Events are generated only inside the same transaction that publishes the corresponding complete snapshot. Cancelled, failed, interrupted, partial, count-mismatched, or mid-enumeration-disappearance scans never enter reconciliation. If publication fails, event inserts roll back with the snapshot.

Rename/move matching can produce false positives when two unrelated files share the same metadata fingerprint and have similar names. Likely_renamed and possible_move events are marked with sub-1.0 confidence and presented as inferences, not confirmed facts.

Filesystem timestamps have platform-dependent precision. Chronicle compares stored metadata literally; therefore a filesystem that rewrites modification time may produce a legitimate metadata-modified event even when content changes are unknown. Chronicle does not read content and does not claim content-level change.

## Milestone 3 watcher events

Watcher events use `event_source = 'watcher'` and confidence `0.85`. They are best-effort native filesystem observations, not a complete audit log. Chronicle validates every raw event path against the authorized root, debounces duplicate events, rechecks current metadata, filters common temporary files, and coalesces rapid save sequences into one final path observation before writing any timeline event.

Watcher classification is path-based:

- final present path without a present row: `created`;
- final present path with changed metadata: `modified`;
- final missing path with a previously present row: `deleted`;
- final missing temp path or path not known to Chronicle: no event.

Confirmed rename or move detection is not implemented. Atomic replacement, save-as, and rename-like patterns may appear as delete plus create, or as modified when the final path is unchanged. Manual and startup reconciliation scans remain the recovery mechanism for missed watcher events.

## Activity sessions

Sessions are inferred groupings of Chronicle file events designed to answer "what did I work on during a sitting?" without tracking applications, windows, or input devices.

### Algorithm

1. Load up to 50 000 recent `file_events`, ordered by `detected_at` then `id`.
2. Group events into sessions using a conservative time-gap threshold (default 30 minutes). Events closer than the gap stay in the same session; a gap larger than the threshold starts a new session.
3. Each session is capped at 100 events. Sessions exceeding the cap are split.
4. No more than 1 000 sessions are generated per run; remaining events are ignored.
5. Project assignment uses a majority vote: each file in the session is mapped to its active project(s), and the project with the most file affiliations is assigned.
6. The title is derived from the assigned project name (if any) plus a timestamp; otherwise a generic "Activity · timestamp" label.
7. Generated sessions replace only `auto`-status sessions. User-edited, accepted, or rejected sessions are preserved.
8. Merge: two sessions can be combined manually, moving all events and files to the target, recomputing the time range, and deleting the source.

### Invariants

- Sessions never read file contents, keyboard input, browser history, window titles, or screenshots.
- Session boundaries are approximate; the time gap heuristic may merge or split real activity.
- Chronicle never calculates productivity scores, focus ratings, or performance metrics.
- Rejecting a session keeps it in the database but removes it from active views without deleting underlying events.
- Merging sessions is idempotent for data: events are never duplicated.
