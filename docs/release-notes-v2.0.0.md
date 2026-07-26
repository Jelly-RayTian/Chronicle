# Chronicle v2.0.0 — Public Release

Chronicle v2.0.0 exists to make the privacy-first desktop application safe to
evaluate outside its development repository. It adds release discipline and
evidence, not a new tracking surface.

## Highlights

- Windows 11 x64 NSIS installer built only after all frontend and Rust gates pass.
- Exact tag-to-manifest validation across npm, Cargo, Cargo.lock, and Tauri.
- Installer PE, size, Authenticode-state, and SHA-256 validation before release creation.
- Real-product screenshots from an isolated profile and temporary authorized folder.
- Keyboard skip navigation, visible focus for all controls, meaningful names, and contained/restored dialog focus.
- Repeatable smoke test and documented privacy, security, accessibility, architecture, database, testing, and release contracts.

## Privacy and file safety

Chronicle has no telemetry, analytics, cloud sync, or hidden monitoring.
Metadata scanning and monitoring stay inside explicitly authorized roots.
Content indexing remains disabled by default and opt-in per folder. Diagnostics
contain counts and status only. Removing a Chronicle index, disabling content
indexing, uninstalling, or clearing Chronicle data does not delete original files.

## Installation warning

The Windows installer is not code-signed. Windows may show an unknown-publisher
or SmartScreen warning. Verify the installer against the published
`SHA256SUMS.txt`. The release workflow reports `NotSigned` honestly and rejects
other invalid signature states.

## Known limitations

Windows 11 x64 with WebView2 is the certified target. Watcher events remain
best-effort; filename case folding and filesystem semantics vary; content
indexing supports configured UTF-8-like text only; heuristics remain
user-reviewable; and there is no cloud sync, AI, auto-update, or cross-platform
installer. See [Known limitations](known-limitations.md).

## Upgrade

Install over an earlier build and launch normally. Database migrations are
ordered and preserve the last complete snapshot. Back up the Chronicle
application-data directory before testing prerelease builds. Original indexed
folders are never used as migration targets.

## Verification

The final Windows installer passed PE/size validation and reported the expected
`NotSigned` state. Its published SHA-256 is
`40243bb3dd7155b82ab074e5a54cb3386a3831ca8d6acc00d9d84c23c70fcd02`.
All required frontend and Rust checks passed, and an isolated install/uninstall
left the real smoke-test fixture hashes unchanged.
