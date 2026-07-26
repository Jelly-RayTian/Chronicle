# v2.0.0 privacy and security audit

Audit scope: React/native boundary, Tauri commands, scanner, watcher, content
indexer, diagnostics, SQLite mutations, platform open/reveal, dependencies,
workflows, and release artifacts.

## Evidence-backed findings

| Question                                  | Result               | Evidence                                                                                                                      |
| ----------------------------------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Telemetry or analytics?                   | Absent               | No telemetry client or analytics endpoint; Settings states the guarantee.                                                     |
| Hidden monitoring?                        | Absent               | Watcher intent is per authorized folder and disabled by default.                                                              |
| Arbitrary scan paths?                     | Rejected             | Scan commands accept a registered folder id; Rust reloads its canonical root.                                                 |
| Content read by default?                  | No                   | Per-folder opt-in is required; extension, exclusion, and 8 MiB hard limits apply.                                             |
| Diagnostics disclose paths/names/content? | No                   | Export model contains version, schema/status, timestamp, and aggregate counts only; Rust tests inspect the serialized report. |
| Destructive original-file operations?     | Absent in production | Removal mutations delete Chronicle SQLite/index rows only. Filesystem deletes found in Rust are test-fixture cleanup.         |
| Unsafe open/reveal?                       | Guarded              | Rust reloads file/root state, rejects absent/symlink/escaped paths, and passes paths as process arguments without a shell.    |
| Release integrity?                        | Improved             | Exact tag/version validation, full gates, PE/size/signature checks, and SHA-256 precede release creation.                     |

## Absence audit

Deliberate absences strengthen Chronicle's privacy boundary: telemetry,
analytics, cloud sync, AI, hidden startup monitoring, productivity scoring, and
destructive file organization. Accidental public-release gaps weakened the
project: real screenshots, release/smoke documentation, exact tag validation,
artifact hashes, unsigned-installer disclosure, and a recorded accessibility
review. v2.0.0 closes those process/presentation gaps. Code signing remains
absent because a trusted certificate and protected signing process are external
release infrastructure; the warning is explicit rather than concealed.

## Residual risks

Watcher APIs are lossy; authorized local content stored in SQLite is sensitive;
an unsigned installer has weaker publisher identity; heuristics can be wrong;
and lockfiles cannot eliminate dependency supply-chain risk. Users should
protect their Windows account, verify hashes, keep content indexing selective,
and reconcile with complete scans after watcher errors.
