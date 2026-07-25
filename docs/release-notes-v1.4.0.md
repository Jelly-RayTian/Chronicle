# Chronicle v1.4.0 — Performance & Scale

Chronicle v1.4.0 exists to prove that its privacy and deletion-safety model is
usable on real folders, not only demonstration-sized fixtures.

## Highlights

- Reproducible release-profile benchmarks now cover 1,000, 10,000, and an
  optional 50,000 generated files.
- The suite separately measures scan staging, atomic reconciliation after 10%
  file moves, timeline query, filename search, watcher burst coalescing, and
  SQLite footprint.
- Scan staging uses 1,024-row transactions, reducing a 10,000-file scan from
  about 40 staging/progress transactions to 10. Repeated wall-clock runs varied,
  so no precise scan-speed claim is made.
- Watcher bursts discard identical raw paths before canonicalization, reducing
  the measured 40,000-notification burst from about 3.23 s to 0.31 s while
  preserving authorized-root checks.
- Large folders show a visible warning at 10,000 observed files.
- Content indexing has an 8 MiB hard per-file cap, enforced by the Rust reader
  and V10 SQLite migration; the default remains 1 MiB.
- Timeline requests remain keyset-paginated and capped at 100 rows; Chronicle's
  UI requests 30 and uses browser rendering containment for off-screen cards.

## Honest limits

On the documented Windows machine, 50,000 files scanned in about 6.65 s and a
10% move reconciliation completed in about 6.06 s. Filesystem, antivirus,
storage, and CPU differences matter. Peak process memory is not reported
because Chronicle does not yet have trustworthy cross-platform instrumentation;
the benchmark reports SQLite bytes instead.

No speculative architecture rewrite, new tracking source, analytics, cloud
service, or unrelated product feature is included.

See [Performance benchmarks](benchmarks.md) for commands, methodology, raw
results, baselines, and environment details.
