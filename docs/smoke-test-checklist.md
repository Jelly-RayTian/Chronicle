# Windows public-release smoke test

Record the commit, installer filename, SHA-256, Windows version, WebView2
version, result, and evidence path. Use a temporary fixture outside the
repository and an isolated Chronicle application-data profile. Never use a
production Chronicle database.

## Installation and first launch

- [ ] Artifact validator passes and SHA-256 matches.
- [ ] Installer shows Chronicle, correct v2.0.0 version, install destination, and uninstaller.
- [ ] Unsigned-publisher warning is expected and documented.
- [ ] First launch opens a real empty Timeline with no fabricated records.
- [ ] English and Simplified Chinese can be selected.

## Core workflow

- [ ] Create a temporary folder with real text/source files; record file hashes.
- [ ] Add only that folder and confirm the authorized root.
- [ ] Run metadata scan; confirm created events and completed scan status.
- [ ] Enable monitoring explicitly; create/modify a fixture file and confirm coalesced events.
- [ ] Search Timeline by filename and filters.
- [ ] Confirm filename search works with content indexing disabled.
- [ ] Opt in to content indexing for the folder and confirm content search.
- [ ] Disable/clear content indexing and confirm original fixture content is unchanged.
- [ ] Remove the Chronicle index and confirm the folder and every original file/hash remain.

## Accessibility and failure states

- [ ] Complete the workflow using keyboard only; the skip link and focus indicator are visible.
- [ ] Dialog focus remains contained, Escape closes, and focus returns to the trigger.
- [ ] Controls have meaningful accessible names.
- [ ] Empty, loading, warning, recoverable error, deleted-file, and unavailable-folder states explain the next action.
- [ ] Light and dark themes meet the reviewed contrast pairs in `docs/accessibility.md`.

## Uninstall

- [ ] Close Chronicle and run its generated uninstaller.
- [ ] Application binaries and shortcuts are removed.
- [ ] Temporary fixture folder and recorded hashes remain unchanged.
- [ ] Record separately whether per-user Chronicle application data remains; remove it only from the isolated profile.

## Demo GIF checklist

- [ ] Capture 1280×800 or similar at 100% scale with no personal paths or notifications.
- [ ] Show add folder → scan → Timeline → search in 20–30 seconds.
- [ ] Show the content-indexing opt-in before any content search.
- [ ] Keep pointer motion deliberate; trim idle time; avoid unreadably fast cuts.
- [ ] Add short captions for “local only”, “opt-in content”, and “original files untouched”.
- [ ] Export an optimized GIF or MP4 and verify text remains legible before attaching to the GitHub Release.
