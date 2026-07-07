# Platform Limitations

Windows 11 and WebView2 are the current installation and visual-verification target. Rust keeps platform behavior behind a dedicated boundary, but macOS and Linux open/reveal paths are not yet manually certified.

Path case rules, timestamp availability/precision, ACLs, junctions, symbolic links, network shares, removable drives, cloud placeholders, and maximum path lengths differ by platform and filesystem. Creation time is nullable. Chronicle compares metadata exactly and cannot determine whether file content changed.

A missing entry or permission failure during enumeration makes the scan incomplete and preserves the prior snapshot. Intentionally skipped symbolic links and non-regular entries are warnings. Network and removable roots can become unavailable between discovery and an explicit open/reveal action; the action revalidates existence, regular-file type, symbolic-link status, and authorized-root containment and may still fail safely.

Filename search relies on SQLite `LIKE`; ASCII case-insensitive matching is reliable, while case folding for all Unicode scripts depends on SQLite behavior. Date filters are converted by the UI from local calendar boundaries to UTC RFC 3339 bounds.

Windows open/reveal uses `explorer.exe`; macOS uses `open`; Linux uses `xdg-open`. Paths are passed as process arguments, never concatenated into a shell command. Chronicle does not automatically launch files.

## Filesystem watching

Milestone 3 uses the Rust `notify` crate over native platform APIs. Operating systems may duplicate, delay, reorder, drop, or coalesce filesystem notifications. Some editors save through temporary files or atomic replacement. Chronicle debounces and rechecks metadata before writing events, but watcher history is not a perfect record.

Watcher paths are validated against the authorized root before processing. Symbolic links and non-regular files are skipped. If a watched folder is removed, unavailable, permission-denied, or an event storm exceeds the bounded pending queue, Chronicle stops publishing watcher events for that folder and keeps the last complete snapshot. Run a metadata scan to reconcile missed changes.

## File identity and rename/move detection

Milestone 4 adds file identity tracking:

- **Unix (Linux/macOS)**: Uses stable `(device, inode)` from `symlink_metadata`. This identifier persists across renames and intra-filesystem moves but changes on cross-filesystem moves.
- **Windows**: Uses a metadata fingerprint (file size + modification time + creation time). This is stable across renames and same-volume moves when the file content is unchanged, but changes if the file is modified AND renamed between scans.
- **Other platforms**: Same metadata fingerprint approach as Windows.

Rename/move detection during reconciliation:

- Confirmed matches (`renamed`, `moved`) require identical identity keys. Confidence: 1.0.
- Heuristic matches (`likely_renamed`, `possible_move`) use same size + close modification time + filename similarity. Confidence: 0.8 / 0.4.
- False positives are possible when two unrelated files share the same metadata fingerprint and similar names. Inferred events are user-reviewable.
- Watcher events do not perform rename/move detection. A rename observed by the watcher may appear as separate deleted and created events, or as a path-based modified event. Run a metadata scan for rename detection.

Rename and move confirmation is not implemented in this milestone. A rename may appear as deleted plus created, and a same-path atomic replacement may appear as modified.

## Version families and manual confirmation

Milestone 5 groups files into version families using metadata-only heuristics:

- **Name-stem similarity**: normalized base names without version tokens must be close (Jaro-Winkler).
- **Version tokens**: extracted numeric tokens (e.g. `v2`, `03`, `2024`) are compared with Jaccard overlap.
- **Folder proximity**: files in the same folder score higher; cross-folder matches are still allowed but confidence is reduced.
- **Identity keys**: on Unix, matching `(device, inode)` strongly supports family membership. On Windows, the metadata fingerprint can falsely match unrelated files of the same size modified in the same second, so identity-key evidence is only counted when the names already share version tokens and the key is a stable `ino:` key.

Suggestions are generated only when the user clicks “Refresh suggestions”. The UI marks suggested families as pending until accepted or rejected. Chronicle never renames, moves, or deletes original files as part of version-family actions. Chronological ordering within a family uses modification timestamps and is approximate; it does not prove which file is the true latest version.

False positives are possible for files with similar names (e.g. `draft.docx` and `final.docx`). Rejecting or removing a member updates the persisted decision but leaves the underlying file and events untouched.

## Project groups and activity sessions

Milestone 6 adds contextual groupings that do not modify original files:

- **Project suggestions** are heuristic groupings based on folder proximity, filename keywords, temporal co-occurrence, confirmed version families, Git repository membership, and user labels. They are generated only on explicit request and remain user-reviewable.
- **Git repository detection** checks for a `.git` directory in ancestor paths within the authorized root using `symlink_metadata`; it does not read Git contents or history. Symbolic-link `.git` directories are not followed.
- **Temporal co-occurrence** groups files with events in the same clock hour. It is a weak signal and is combined with stronger signals before presentation.
- **Activity sessions** are inferred only from Chronicle file events. Sessions split on time gaps and cap event counts. Boundaries are approximate and may merge or separate real activity depending on the chosen gap threshold.
- **No productivity metrics**: sessions do not produce scores, rankings, focus ratings, or any measure of user productivity. The UI shows only event counts, file lists, and optional project links.

Rejecting a project or session removes it from active views but does not delete underlying files or events. Edited session titles and project assignments are stored as user decisions.

## Optional local full-text indexing

Milestone 7 adds privacy-first content search:

- **Opt-in per folder**: content indexing is disabled by default and must be explicitly enabled for each indexed folder.
- **Supported formats**: `.txt`, `.md`, and a configurable list of source-code extensions. Files outside the allowed list are ignored.
- **Size limits**: each file must be at or below the configured `content_indexing_max_bytes` (default 1 MiB). Larger files are skipped.
- **Secret exclusions**: default patterns exclude `.env`, `.env.*`, `*.key`, `*.pem`, `*.crt`, `*.p12`, `*.pfx`, `id_rsa`, `id_ed25519`, `id_ecdsa`, `.htpasswd`, `.npmrc`, `.pypirc`, and `netrc`. Users can edit the exclusion list.
- **Local-only storage**: extracted text lives in the same local SQLite database as metadata, using FTS5. Nothing is uploaded or sent to a remote service.
- **No binary extraction**: only lossy UTF-8 text is read. PDF, DOCX, images, and other binary formats are not parsed.
- **Incremental sync**: after a complete scan or watcher batch, the indexer reconciles the current snapshot with the previous index. Disabling a folder or clearing the index removes the corresponding rows.
- **Snippet safety**: FTS5 snippets are produced with custom markers, then stripped and HTML-escaped before serialization. The UI never inserts them as raw HTML.
- **Filename search**: works for all indexed folders regardless of content-indexing state.
