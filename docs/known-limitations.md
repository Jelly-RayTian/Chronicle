# Known limitations

This document summarizes the current known limitations of Chronicle v1.0.1. It is honest about what the current release does not guarantee so users and contributors can set the right expectations.

## Platform support

- The primary target is **Windows 11 with WebView2 Runtime**. macOS and Linux paths are isolated behind a platform boundary but are not manually certified in this release candidate.
- Path case rules, timestamp precision, ACLs, junctions, symbolic links, network shares, removable drives, cloud placeholder files, and maximum path lengths differ by filesystem. Chronicle compares metadata exactly and cannot determine whether file content changed.
- A missing entry or permission failure during enumeration makes a scan incomplete and preserves the prior snapshot. Symbolic links and non-regular entries are intentionally skipped and recorded as warnings.
- Network and removable roots can become unavailable between discovery and an explicit open/reveal action. The action revalidates existence, regular-file type, symbolic-link status, and authorized-root containment and may still fail safely.

## Search

- Filename search uses SQLite `LIKE`. ASCII case-insensitive matching is reliable; full Unicode case folding depends on SQLite behavior.
- Date filters are converted by the UI from local calendar boundaries to UTC RFC 3339 bounds.
- Content indexing is limited to lossy UTF-8 text in `.txt`, `.md`, and configured source-code extensions. PDF, DOCX, images, and other binary formats are not parsed.

## Filesystem watching

- Chronicle uses the Rust `notify` crate over native platform APIs. Operating systems may duplicate, delay, reorder, drop, or coalesce filesystem notifications.
- Some editors save through temporary files or atomic replacement. Chronicle debounces and rechecks metadata before writing events, but watcher history is not a perfect record.
- Watcher paths are validated against the authorized root before processing. Symbolic links and non-regular files are skipped.
- If a watched folder is removed, unavailable, permission-denied, or an event storm exceeds the bounded pending queue, Chronicle stops publishing watcher events for that folder and keeps the last complete snapshot. Run a metadata scan to reconcile missed changes.

## File identity and rename/move detection

- **Unix (Linux/macOS)**: identity uses stable `(device, inode)` from `symlink_metadata`. It persists across renames and intra-filesystem moves but changes on cross-filesystem moves.
- **Windows**: identity uses a metadata fingerprint (file size + modification time + creation time). It is stable across renames and same-volume moves when content is unchanged, but may change if the file is modified and renamed between scans.
- Confirmed `renamed`/`moved` events require identical identity keys.
- Heuristic `likely_renamed`/`possible_move` events use same size + close modification time + filename similarity. False positives are possible when unrelated files share similar metadata and names.
- Watcher events do not perform rename/move detection. A rename observed by the watcher may appear as separate deleted and created events, or as a path-based modified event. Run a metadata scan for rename detection.

## Version families

- Version-family suggestions are metadata-only heuristics (name-stem similarity, version-token overlap, folder proximity, identity keys). They can produce false positives for files with similar names.
- Chronological ordering within a family uses modification timestamps and is approximate; it does not prove which file is the true latest version.
- Accepting, rejecting, splitting, merging, or removing a member updates persisted decisions but never renames, moves, or deletes original files.

## Projects and sessions

- Project suggestions are heuristic groupings based on folder proximity, filename keywords, temporal co-occurrence, confirmed version families, Git repository membership, and user labels. They are user-reviewable, not authoritative.
- Git repository detection checks for a `.git` directory in ancestor paths within the authorized root using `symlink_metadata`; it does not read Git contents or history.
- Activity sessions are inferred only from Chronicle file events. Boundaries are approximate and may merge or separate real activity depending on the chosen gap threshold.
- Chronicle does not calculate productivity scores, rankings, focus ratings, or any other measure of user productivity.

## Content indexing

- Content indexing is opt-in per folder. By default, Chronicle does not open file contents.
- Indexed contents are stored only in the local SQLite database and are never uploaded or analyzed by AI.
- Secret-file exclusions are provided as defaults; users should review and adjust them for their environment.
- Clearing the content index or disabling content indexing for a folder removes indexed rows but does not delete original files.

## Diagnostics

- The diagnostics export includes application counts and status only. It intentionally does not contain file paths, filenames, or file contents so it can be shared safely.

## Error messages

- Error messages include file-safety reassurances and guidance where applicable. However, they cannot validate every local filesystem state; for complex failures, the safest action is always to remove the Chronicle index and re-add the folder--this never deletes original files.
