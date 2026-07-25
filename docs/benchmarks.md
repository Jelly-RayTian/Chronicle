# Performance benchmarks

Chronicle v1.4.0 includes an opt-in, reproducible Rust benchmark suite for large
indexed folders and long event histories. The suite creates every fixture in a
temporary directory and opens a temporary SQLite database. Fixture creation is
excluded from scan timing, and no production Chronicle data is read or changed.

## Reproduce

Run the optimized build serially:

```powershell
$env:CHRONICLE_BENCH_COUNTS='1000,10000'
cargo test --release --manifest-path src-tauri/Cargo.toml performance::tests::benchmark_large_folders_and_long_histories -- --ignored --nocapture --test-threads=1
```

The default counts are `1,000,10,000`. The optional 50,000-file run is:

```powershell
$env:CHRONICLE_BENCH_COUNTS='50000'
cargo test --release --manifest-path src-tauri/Cargo.toml performance::tests::benchmark_large_folders_and_long_histories -- --ignored --nocapture --test-threads=1
```

Each fixture has 100 deterministic subdirectories and zero-padded text
filenames. Scan time measures traversal plus staging, excluding fixture creation
and final publication. Reconciliation measures only atomic publication after
10% of files are moved to another directory. Timeline and filename search are
warmed up and report the median of 21 queries. The watcher benchmark sends ten
duplicates per path and reports raw-to-unique coalescing time. Database bytes
are the SQLite file size after the run.

## Environment

Measurements below were captured on 2026-07-25:

- Windows 11 Pro 10.0.26200;
- Intel Core i5-12600KF, 10 cores / 16 logical processors;
- 31.8 GiB RAM;
- Rust 1.96.0;
- Chronicle release profile, one benchmark thread;
- local Windows filesystem and temporary directories.

Results will differ with filesystem, antivirus, storage, CPU, and background
load. These figures are evidence from this machine, not universal guarantees.

## v1.4.0 results

|  Files |         Scan | Reconciliation (10% moved) | Timeline median | Filename search median |  Watcher burst | Watcher coalescing |  SQLite size |
| -----: | -----------: | -------------------------: | --------------: | ---------------------: | -------------: | -----------------: | -----------: |
|  1,000 |    69.845 ms |                  20.134 ms |        0.193 ms |               0.112 ms | 10,000 → 1,000 |          43.396 ms |  1,687,552 B |
| 10,000 | 1,180.743 ms |                 769.680 ms |        0.406 ms |               0.273 ms | 40,000 → 4,000 |         310.686 ms | 14,692,352 B |
| 50,000 | 6,653.796 ms |               6,061.968 ms |        0.162 ms |               0.209 ms | 40,000 → 4,000 |         354.319 ms | 67,788,800 B |

The 50,000-file result was practical on this machine, but scan and
reconciliation scaling is not perfectly linear. It should be treated as a
tested upper sample, not a promise that every 50,000-file tree completes in the
same time.

Repeated final-source 10,000-file runs showed meaningful filesystem timing
variance: scan ranged from 898.776 to 1,362.346 ms and reconciliation from
612.974 to 830.009 ms. The table uses the most recent run rather than selecting
the fastest result. Watcher coalescing was much more stable at 306.267 to
318.974 ms.

## Measured changes

The clear pre-change bottleneck was watcher duplicate-path validation. The
10,000-file baseline used the same release command and fixture:

| Metric                       |     Baseline | v1.4.0 repeated range |            Result |
| ---------------------------- | -----------: | --------------------: | ----------------: |
| Watcher 40,000 → 4,000 burst | 3,228.233 ms |    306.267–318.974 ms | 10.1–10.5× faster |

The bottleneck was repeated canonicalization before duplicate paths were
discarded. v1.4.0 first removes identical raw paths, then validates and
canonicalizes each unique path. Authorized-root validation remains in place.

Scan staging grew from 256 to 1,024 rows per transaction. At 10,000 files this
reduces staging/progress transactions from approximately 40 to 10 while
retaining cancellation points. Because measured wall-clock scan and
reconciliation timings varied across repeated final-source runs, this document
does not claim a precise latency improvement from batching.

Timeline and filtered filename search remained below 0.5 ms at 10,000
files, so v1.4.0 does not add speculative query indexes or rewrite those SQL
paths. Timeline remains keyset-paginated and capped at 100 rows per native
request; the UI requests 30. Browser-native `content-visibility` containment
skips off-screen timeline and search card layout/paint work.

## Memory and limits

The suite reports SQLite file size as a reproducible storage-footprint proxy.
Peak process memory is not reported: stable cross-platform resident-memory
instrumentation is not available in the current dependency set, and invented
or incomparable values would be misleading.

Safety limits in v1.4.0:

- large-folder UI warning at 10,000 observed files;
- 8 MiB hard maximum per content-indexed file (1 MiB default);
- timeline native page size capped at 100 rows (30 requested by the UI);
- filename/content search capped at 200 rows (50 requested by the UI);
- watcher pending queue capped at 4,096 unique paths; exceeding it stops
  publication and requires a complete reconciliation scan.
