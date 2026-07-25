#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };

    use tempfile::TempDir;

    use crate::{
        content_indexing, database::Database, folders::register_folder, models::TimelineRequest,
        scanner, watcher,
    };

    const DEFAULT_COUNTS: &[usize] = &[1_000, 10_000];
    const QUERY_SAMPLES: usize = 21;
    const WATCHER_DUPLICATES_PER_PATH: usize = 10;

    #[derive(Debug)]
    struct BenchmarkResult {
        files: usize,
        scan: Duration,
        reconciliation: Duration,
        timeline_median: Duration,
        search_median: Duration,
        watcher_burst_events: usize,
        watcher_unique_paths: usize,
        watcher_coalescing: Duration,
        database_bytes: u64,
    }

    fn benchmark_counts() -> Vec<usize> {
        let Some(raw) = std::env::var_os("CHRONICLE_BENCH_COUNTS") else {
            return DEFAULT_COUNTS.to_vec();
        };
        let parsed = raw
            .to_string_lossy()
            .split(',')
            .filter_map(|value| value.trim().parse::<usize>().ok())
            .filter(|count| *count > 0)
            .collect::<Vec<_>>();
        if parsed.is_empty() {
            DEFAULT_COUNTS.to_vec()
        } else {
            parsed
        }
    }

    fn create_tree(root: &Path, file_count: usize) {
        for bucket in 0..100 {
            let directory = root.join(format!("bucket_{bucket:02}"));
            fs::create_dir(&directory)
                .unwrap_or_else(|error| panic!("create fixture directory {directory:?}: {error}"));
        }
        for index in 0..file_count {
            let directory = root.join(format!("bucket_{:02}", index % 100));
            let path = directory.join(format!("file_{index:06}.txt"));
            fs::write(&path, format!("chronicle benchmark fixture {index}\n"))
                .unwrap_or_else(|error| panic!("write fixture {path:?}: {error}"));
        }
    }

    fn start_and_traverse(
        root: &Path,
        database: &Database,
        folder_id: i64,
    ) -> (i64, scanner::ScanCounts, Duration) {
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("start scan: {error}"));
        let started = Instant::now();
        let counts = scanner::traverse(
            database,
            run.scan_run_id,
            folder_id,
            root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("traverse fixture: {error}"));
        (run.scan_run_id, counts, started.elapsed())
    }

    fn move_fixture_subset(root: &Path, file_count: usize) {
        let moved_root = root.join("moved");
        fs::create_dir(&moved_root)
            .unwrap_or_else(|error| panic!("create moved fixture directory: {error}"));
        for index in (0..file_count).step_by(10) {
            let source = root
                .join(format!("bucket_{:02}", index % 100))
                .join(format!("file_{index:06}.txt"));
            let target = moved_root.join(format!("file_{index:06}.txt"));
            fs::rename(&source, &target)
                .unwrap_or_else(|error| panic!("move fixture {source:?} to {target:?}: {error}"));
        }
    }

    fn complete(
        database: &Database,
        folder_id: i64,
        run_id: i64,
        counts: scanner::ScanCounts,
    ) -> Duration {
        let started = Instant::now();
        database
            .complete_scan(
                run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("complete scan: {error}"));
        started.elapsed()
    }

    fn median(mut samples: Vec<Duration>) -> Duration {
        samples.sort_unstable();
        samples[samples.len() / 2]
    }

    fn measure_queries(database: &Database, folder_id: i64) -> (Duration, Duration) {
        let timeline_request = TimelineRequest::first_page(50);
        let search_request = crate::models::SearchFilesRequest {
            query: "file_009".to_owned(),
            mode: crate::models::ContentSearchMode::Filename,
            folder_id: Some(folder_id),
            extension: Some("txt".to_owned()),
            date_from: None,
            date_to: None,
            presence: Some(crate::models::PresenceFilter::Present),
            event_type: None,
            limit: 50,
        };

        let _ = database
            .query_timeline_page(&timeline_request)
            .unwrap_or_else(|error| panic!("warm timeline query: {error}"));
        let _ = content_indexing::search_files(database, search_request.clone())
            .unwrap_or_else(|error| panic!("warm filename search: {error}"));

        let timeline = median(
            (0..QUERY_SAMPLES)
                .map(|_| {
                    let started = Instant::now();
                    let page = database
                        .query_timeline_page(&timeline_request)
                        .unwrap_or_else(|error| panic!("timeline query: {error}"));
                    assert_eq!(page.items.len(), 50);
                    started.elapsed()
                })
                .collect(),
        );
        let search = median(
            (0..QUERY_SAMPLES)
                .map(|_| {
                    let started = Instant::now();
                    let _ = content_indexing::search_files(database, search_request.clone())
                        .unwrap_or_else(|error| panic!("filename search: {error}"));
                    started.elapsed()
                })
                .collect(),
        );
        (timeline, search)
    }

    fn measure_watcher_burst(root: &Path, file_count: usize) -> (usize, usize, Duration) {
        let unique_count = file_count.min(4_000);
        let mut burst = Vec::with_capacity(unique_count * WATCHER_DUPLICATES_PER_PATH);
        for duplicate in 0..WATCHER_DUPLICATES_PER_PATH {
            for index in 0..unique_count {
                let bucket = index % 100;
                let path = root
                    .join(format!("bucket_{bucket:02}"))
                    .join(format!("file_{index:06}.txt"));
                if duplicate % 2 == 0 {
                    burst.push(path);
                } else {
                    burst.push(PathBuf::from(path.to_string_lossy().into_owned()));
                }
            }
        }
        let started = Instant::now();
        let coalesced = watcher::coalesce_paths_for_test(root, &burst)
            .unwrap_or_else(|error| panic!("coalesce watcher burst: {error}"));
        let elapsed = started.elapsed();
        assert_eq!(coalesced.len(), unique_count);
        (burst.len(), coalesced.len(), elapsed)
    }

    fn run_with(count: usize) -> BenchmarkResult {
        let directory = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("create root: {error}"));
        create_tree(&root, count);
        let database_path = directory.path().join("chronicle.sqlite3");
        let database =
            Database::open(&database_path).unwrap_or_else(|error| panic!("open database: {error}"));
        let folder = register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("register root: {error}"))
            .folder;

        let (run_id, counts, scan) = start_and_traverse(&root, &database, folder.id);
        let _initial_publication = complete(&database, folder.id, run_id, counts);
        assert_eq!(
            database
                .list_file_records(folder.id)
                .unwrap_or_else(|error| panic!("list indexed files: {error}"))
                .len(),
            count
        );

        move_fixture_subset(&root, count);
        let (reconcile_run_id, reconcile_counts, _second_traversal) =
            start_and_traverse(&root, &database, folder.id);
        let reconciliation = complete(&database, folder.id, reconcile_run_id, reconcile_counts);
        let (timeline_median, search_median) = measure_queries(&database, folder.id);
        let (watcher_burst_events, watcher_unique_paths, watcher_coalescing) =
            measure_watcher_burst(&root, count);
        let database_bytes = fs::metadata(database_path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        BenchmarkResult {
            files: count,
            scan,
            reconciliation,
            timeline_median,
            search_median,
            watcher_burst_events,
            watcher_unique_paths,
            watcher_coalescing,
            database_bytes,
        }
    }

    fn milliseconds(duration: Duration) -> f64 {
        duration.as_secs_f64() * 1_000.0
    }

    #[test]
    #[ignore = "reproducible performance suite; run explicitly with --ignored --nocapture"]
    fn benchmark_large_folders_and_long_histories() {
        println!(
            "files,scan_ms,reconciliation_ms,timeline_median_ms,search_median_ms,watcher_raw,watcher_unique,watcher_coalescing_ms,database_bytes"
        );
        for count in benchmark_counts() {
            let result = run_with(count);
            println!(
                "{},{:.3},{:.3},{:.3},{:.3},{},{},{:.3},{}",
                result.files,
                milliseconds(result.scan),
                milliseconds(result.reconciliation),
                milliseconds(result.timeline_median),
                milliseconds(result.search_median),
                result.watcher_burst_events,
                result.watcher_unique_paths,
                milliseconds(result.watcher_coalescing),
                result.database_bytes,
            );
        }
    }
}
