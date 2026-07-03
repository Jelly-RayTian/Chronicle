use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::SystemTime,
};

use chrono::{DateTime, SecondsFormat, Utc};

use crate::{
    database::Database,
    errors::ChronicleError,
    identity,
    models::{AvailabilityStatus, ScanTaskSnapshot, TaskStatus},
    platform::{classify_io_error, path_to_string},
    tasks::ScanTaskManager,
};

const BATCH_SIZE: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFile {
    pub normalized_path: String,
    pub name: String,
    pub parent_path: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub filesystem_created_at: Option<String>,
    pub filesystem_modified_at: String,
    pub identity_key: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ScanCounts {
    pub(crate) files_seen: u64,
    pub(crate) warnings: u64,
    pub(crate) errors: u64,
}

fn system_time(value: SystemTime) -> String {
    DateTime::<Utc>::from(value).to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn failure_kind(error: &ChronicleError) -> &'static str {
    match error {
        ChronicleError::MissingOrMoved => "missing_or_moved",
        ChronicleError::PermissionDenied => "permission_denied",
        ChronicleError::Cancelled => "cancelled",
        ChronicleError::Database(_)
        | ChronicleError::Migration(_)
        | ChronicleError::DatabaseState => "database",
        _ => "inaccessible",
    }
}

fn availability_for_error(error: &ChronicleError) -> AvailabilityStatus {
    match error {
        ChronicleError::MissingOrMoved => AvailabilityStatus::MissingOrMoved,
        ChronicleError::PermissionDenied => AvailabilityStatus::PermissionDenied,
        _ => AvailabilityStatus::Inaccessible,
    }
}

fn check_cancelled(token: &AtomicBool) -> Result<(), ChronicleError> {
    if token.load(Ordering::Acquire) {
        Err(ChronicleError::Cancelled)
    } else {
        Ok(())
    }
}

pub(crate) fn discovered_file(
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<DiscoveredFile, ChronicleError> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ChronicleError::PathEncoding)?
        .to_owned();
    let parent_path = path
        .parent()
        .ok_or(ChronicleError::PathEncoding)
        .and_then(path_to_string)?;
    let modified = metadata.modified().map_err(classify_io_error)?;
    let identity_key = identity::resolve_file_identity(path)
        .ok()
        .and_then(|id| id.as_key().map(str::to_owned));
    Ok(DiscoveredFile {
        normalized_path: path_to_string(path)?,
        name,
        parent_path,
        extension: path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_owned),
        size_bytes: metadata.len(),
        filesystem_created_at: metadata.created().ok().map(system_time),
        filesystem_modified_at: system_time(modified),
        identity_key,
    })
}

fn flush_batch(
    database: &Database,
    scan_run_id: i64,
    folder_id: i64,
    batch: &mut Vec<DiscoveredFile>,
    counts: ScanCounts,
) -> Result<(), ChronicleError> {
    database.stage_scan_batch(
        scan_run_id,
        folder_id,
        batch,
        counts.files_seen,
        counts.warnings,
        counts.errors,
    )?;
    batch.clear();
    Ok(())
}

pub(crate) fn traverse(
    database: &Database,
    scan_run_id: i64,
    folder_id: i64,
    root: &Path,
    cancellation: &AtomicBool,
) -> Result<ScanCounts, ChronicleError> {
    let root_metadata = fs::symlink_metadata(root).map_err(classify_io_error)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(ChronicleError::Inaccessible);
    }

    let mut counts = ScanCounts::default();
    let mut stack = vec![root.to_path_buf()];
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    while let Some(directory) = stack.pop() {
        check_cancelled(cancellation)?;
        let canonical_directory = PathBuf::from(path_to_string(
            &fs::canonicalize(&directory).map_err(classify_io_error)?,
        )?);
        if !canonical_directory.starts_with(root) {
            counts.warnings += 1;
            continue;
        }
        let entries = fs::read_dir(&directory).map_err(classify_io_error)?;
        for entry_result in entries {
            check_cancelled(cancellation)?;
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(ChronicleError::MissingOrMoved);
                }
                Err(error) => return Err(classify_io_error(error)),
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(ChronicleError::MissingOrMoved);
                }
                Err(error) => return Err(classify_io_error(error)),
            };
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                counts.warnings += 1;
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() {
                counts.warnings += 1;
                continue;
            }
            match discovered_file(&path, &metadata) {
                Ok(file) => {
                    counts.files_seen += 1;
                    batch.push(file);
                }
                Err(ChronicleError::MissingOrMoved) => {
                    counts.warnings += 1;
                }
                Err(error) => return Err(error),
            }
            if batch.len() >= BATCH_SIZE {
                flush_batch(database, scan_run_id, folder_id, &mut batch, counts)?;
            }
        }
    }
    flush_batch(database, scan_run_id, folder_id, &mut batch, counts)?;
    check_cancelled(cancellation)?;
    Ok(counts)
}

pub fn start_scan(
    database: Database,
    tasks: ScanTaskManager,
    folder_id: i64,
) -> Result<ScanTaskSnapshot, ChronicleError> {
    let folder = database.get_indexed_folder(folder_id)?;
    let snapshot = database.create_scan_run(folder_id)?;
    let cancellation = match tasks.register(snapshot.scan_run_id) {
        Ok(cancellation) => cancellation,
        Err(error) => {
            let _ = database.fail_scan(
                snapshot.scan_run_id,
                TaskStatus::Failed,
                "database",
                0,
                0,
                1,
            );
            return Err(error);
        }
    };
    let run_id = snapshot.scan_run_id;
    tauri::async_runtime::spawn_blocking(move || {
        let root = PathBuf::from(&folder.normalized_path);
        let result = traverse(&database, run_id, folder_id, &root, &cancellation);
        match result {
            Ok(counts) => {
                let completion = check_cancelled(&cancellation).and_then(|()| {
                    database.complete_scan(
                        run_id,
                        folder_id,
                        counts.files_seen,
                        counts.warnings,
                        counts.errors,
                    )
                });
                if completion.is_err() {
                    let _ = database.fail_scan(
                        run_id,
                        TaskStatus::Failed,
                        "database",
                        counts.files_seen,
                        counts.warnings,
                        counts.errors.saturating_add(1),
                    );
                }
            }
            Err(error) => {
                let cancelled = matches!(error, ChronicleError::Cancelled);
                let status = if cancelled {
                    TaskStatus::Cancelled
                } else {
                    TaskStatus::Failed
                };
                let persisted = database.scan_task_snapshot(run_id).ok();
                let _ = database.fail_scan(
                    run_id,
                    status,
                    failure_kind(&error),
                    persisted.as_ref().map_or(0, |scan| scan.files_seen),
                    persisted.as_ref().map_or(0, |scan| scan.warning_count),
                    persisted.as_ref().map_or(u64::from(!cancelled), |scan| {
                        scan.error_count.saturating_add(u64::from(!cancelled))
                    }),
                );
                if !cancelled {
                    let _ = database
                        .update_folder_availability(folder_id, availability_for_error(&error));
                }
            }
        }
        tasks.finish(run_id);
    });
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::atomic::AtomicBool};

    use tempfile::TempDir;

    use crate::{database::Database, folders::register_folder, models::TaskStatus};

    use super::{discovered_file, traverse};

    fn setup() -> (TempDir, Database, std::path::PathBuf, i64, i64) {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("root should register: {error}"))
            .folder;
        let run = database
            .create_scan_run(folder.id)
            .unwrap_or_else(|error| panic!("scan should start: {error}"));
        (directory, database, root, folder.id, run.scan_run_id)
    }

    #[test]
    fn recursively_collects_metadata_without_changing_contents() {
        let (_directory, database, root, folder_id, run_id) = setup();
        let nested = root.join("nested");
        fs::create_dir(&nested)
            .unwrap_or_else(|error| panic!("nested directory should exist: {error}"));
        let file = nested.join("report.TXT");
        fs::write(&file, b"content must remain private and unchanged")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let before =
            fs::read(&file).unwrap_or_else(|error| panic!("test should read fixture: {error}"));
        let counts = traverse(&database, run_id, folder_id, &root, &AtomicBool::new(false))
            .unwrap_or_else(|error| panic!("scan should succeed: {error}"));
        database
            .complete_scan(
                run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("snapshot should publish: {error}"));
        let records = database
            .list_file_records(folder_id)
            .unwrap_or_else(|error| panic!("snapshot should load: {error}"));
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "report.TXT");
        assert_eq!(records[0].extension.as_deref(), Some("TXT"));
        assert_eq!(
            records[0].size_bytes,
            i64::try_from(before.len()).unwrap_or_default()
        );
        assert_eq!(fs::read(&file).unwrap_or_default(), before);
    }

    #[test]
    fn empty_folders_publish_an_empty_snapshot() {
        let (_directory, database, root, folder_id, run_id) = setup();
        let counts = traverse(&database, run_id, folder_id, &root, &AtomicBool::new(false))
            .unwrap_or_else(|error| panic!("empty scan should succeed: {error}"));
        database
            .complete_scan(
                run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("empty snapshot should publish: {error}"));
        assert!(
            database
                .list_file_records(folder_id)
                .unwrap_or_default()
                .is_empty()
        );
    }

    #[test]
    fn cancellation_and_failed_runs_preserve_the_last_complete_snapshot() {
        let (_directory, database, root, folder_id, first_run_id) = setup();
        fs::write(root.join("stable.txt"), b"stable")
            .unwrap_or_else(|error| panic!("fixture should be written: {error}"));
        let first = traverse(
            &database,
            first_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("first scan should succeed: {error}"));
        database
            .complete_scan(
                first_run_id,
                folder_id,
                first.files_seen,
                first.warnings,
                first.errors,
            )
            .unwrap_or_else(|error| panic!("first snapshot should publish: {error}"));
        let before = database.list_file_records(folder_id).unwrap_or_default();

        let cancelled_run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("second scan should start: {error}"));
        let cancelled = AtomicBool::new(true);
        assert!(
            traverse(
                &database,
                cancelled_run.scan_run_id,
                folder_id,
                &root,
                &cancelled
            )
            .is_err()
        );
        database
            .cancel_scan(cancelled_run.scan_run_id)
            .unwrap_or_else(|error| panic!("cancellation should persist: {error}"));
        assert!(
            database
                .complete_scan(cancelled_run.scan_run_id, folder_id, 0, 0, 0)
                .is_err()
        );
        assert_eq!(
            database
                .scan_task_snapshot(cancelled_run.scan_run_id)
                .unwrap_or_else(|error| panic!("cancelled scan should load: {error}"))
                .status,
            TaskStatus::Cancelled
        );
        assert_eq!(
            database.list_file_records(folder_id).unwrap_or_default(),
            before
        );

        let failed_run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("third scan should start: {error}"));
        database
            .fail_scan(
                failed_run.scan_run_id,
                TaskStatus::Failed,
                "inaccessible",
                0,
                0,
                1,
            )
            .unwrap_or_else(|error| panic!("failure should persist: {error}"));
        assert_eq!(
            database.list_file_records(folder_id).unwrap_or_default(),
            before
        );
    }

    #[test]
    fn missing_roots_fail_without_replacing_the_snapshot() {
        let (_directory, database, root, folder_id, run_id) = setup();
        fs::remove_dir(&root).unwrap_or_else(|error| panic!("root should disappear: {error}"));
        assert!(traverse(&database, run_id, folder_id, &root, &AtomicBool::new(false)).is_err());
        assert!(
            database
                .list_file_records(folder_id)
                .unwrap_or_default()
                .is_empty()
        );
    }

    #[test]
    fn symbolic_links_are_not_followed_when_the_platform_allows_creating_one() {
        let (_directory, database, root, folder_id, run_id) = setup();
        let outside = root.parent().unwrap_or(&root).join("outside.txt");
        fs::write(&outside, b"outside")
            .unwrap_or_else(|error| panic!("outside fixture should exist: {error}"));
        let link = root.join("link.txt");
        #[cfg(windows)]
        let linked = std::os::windows::fs::symlink_file(&outside, &link).is_ok();
        #[cfg(unix)]
        let linked = std::os::unix::fs::symlink(&outside, &link).is_ok();
        #[cfg(not(any(windows, unix)))]
        let linked = false;
        if !linked {
            return;
        }
        let counts = traverse(&database, run_id, folder_id, &root, &AtomicBool::new(false))
            .unwrap_or_else(|error| panic!("scan should skip the link: {error}"));
        database
            .complete_scan(
                run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("snapshot should publish: {error}"));
        assert!(
            database
                .list_file_records(folder_id)
                .unwrap_or_default()
                .is_empty()
        );
        assert_eq!(counts.warnings, 1);
    }

    #[test]
    fn successful_scans_create_and_delete_while_retaining_history() {
        let (_directory, database, root, folder_id, first_run_id) = setup();
        let transient = root.join("transient.txt");
        fs::write(&transient, b"temporary")
            .unwrap_or_else(|error| panic!("fixture should be written: {error}"));
        let first = traverse(
            &database,
            first_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("first scan should succeed: {error}"));
        database
            .complete_scan(
                first_run_id,
                folder_id,
                first.files_seen,
                first.warnings,
                first.errors,
            )
            .unwrap_or_else(|error| panic!("first snapshot should publish: {error}"));
        fs::remove_file(&transient)
            .unwrap_or_else(|error| panic!("fixture should disappear: {error}"));
        let second_run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("second scan should start: {error}"));
        let second = traverse(
            &database,
            second_run.scan_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("second scan should succeed: {error}"));
        database
            .complete_scan(
                second_run.scan_run_id,
                folder_id,
                second.files_seen,
                second.warnings,
                second.errors,
            )
            .unwrap_or_else(|error| panic!("second snapshot should publish: {error}"));
        let records = database.list_file_records(folder_id).unwrap_or_default();
        assert_eq!(records.len(), 1);
        assert!(!records[0].is_present);
        let timeline = database
            .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("timeline should load: {error}"));
        assert_eq!(timeline.items.len(), 2);
        assert_eq!(timeline.items[0].event.event_type, "deleted");
        assert_eq!(timeline.items[1].event.event_type, "created");
        let history = database
            .file_event_history(records[0].id, 100)
            .unwrap_or_else(|error| panic!("history should load: {error}"));
        assert_eq!(history.len(), 2);
        assert!(database.present_file_context(records[0].id).is_err());
        let scans = database
            .scan_history(folder_id, 10)
            .unwrap_or_else(|error| panic!("scan history should load: {error}"));
        assert_eq!(scans.len(), 2);
        assert!(
            scans
                .iter()
                .all(|scan| scan.status == TaskStatus::Completed)
        );
    }

    #[test]
    fn unchanged_rescan_creates_no_duplicate_and_multiple_changes_reconcile() {
        let (_directory, database, root, folder_id, first_run_id) = setup();
        fs::write(root.join("stable.txt"), b"same").unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("change.txt"), b"old").unwrap_or_else(|e| panic!("{e}"));
        let first = traverse(
            &database,
            first_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        database
            .complete_scan(
                first_run_id,
                folder_id,
                first.files_seen,
                first.warnings,
                first.errors,
            )
            .unwrap_or_else(|e| panic!("{e}"));

        let unchanged_run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|e| panic!("{e}"));
        let unchanged = traverse(
            &database,
            unchanged_run.scan_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        database
            .complete_scan(
                unchanged_run.scan_run_id,
                folder_id,
                unchanged.files_seen,
                unchanged.warnings,
                unchanged.errors,
            )
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            database
                .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
                .unwrap_or_else(|e| panic!("{e}"))
                .items
                .len(),
            2
        );

        fs::write(root.join("change.txt"), b"new and larger").unwrap_or_else(|e| panic!("{e}"));
        fs::remove_file(root.join("stable.txt")).unwrap_or_else(|e| panic!("{e}"));
        fs::write(root.join("created.md"), b"new").unwrap_or_else(|e| panic!("{e}"));
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|e| panic!("{e}"));
        let counts = traverse(
            &database,
            run.scan_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        database
            .complete_scan(
                run.scan_run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|e| panic!("{e}"));
        let page = database
            .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
            .unwrap_or_else(|e| panic!("{e}"));
        let kinds = page
            .items
            .iter()
            .take(3)
            .map(|item| item.event.event_type.as_str())
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"created"));
        assert!(kinds.contains(&"modified"));
        assert!(kinds.contains(&"deleted"));
    }

    #[test]
    fn partial_publication_mismatch_rolls_back_snapshot_and_events() {
        let (_directory, database, root, folder_id, run_id) = setup();
        fs::write(root.join("only.txt"), b"one").unwrap_or_else(|error| panic!("{error}"));
        let counts = traverse(&database, run_id, folder_id, &root, &AtomicBool::new(false))
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(
            database
                .complete_scan(
                    run_id,
                    folder_id,
                    counts.files_seen + 1,
                    counts.warnings,
                    counts.errors
                )
                .is_err()
        );
        assert!(
            database
                .list_file_records(folder_id)
                .unwrap_or_default()
                .is_empty()
        );
        assert!(
            database
                .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
                .unwrap_or_else(|error| panic!("{error}"))
                .items
                .is_empty()
        );
    }

    #[test]
    fn timeline_paginates_and_filters_in_sqlite() {
        let (_directory, database, root, folder_id, run_id) = setup();
        fs::write(root.join("alpha.pdf"), b"pdf").unwrap_or_else(|error| panic!("{error}"));
        fs::write(root.join("beta.txt"), b"text").unwrap_or_else(|error| panic!("{error}"));
        let counts = traverse(&database, run_id, folder_id, &root, &AtomicBool::new(false))
            .unwrap_or_else(|error| panic!("{error}"));
        database
            .complete_scan(
                run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("{error}"));

        let mut paged = crate::models::TimelineRequest::first_page(1);
        let first = database
            .query_timeline_page(&paged)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(first.items.len(), 1);
        assert!(first.has_more);
        paged.cursor = first.next_cursor;
        let second = database
            .query_timeline_page(&paged)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(second.items.len(), 1);
        assert_ne!(first.items[0].event.id, second.items[0].event.id);

        let mut filtered = crate::models::TimelineRequest::first_page(50);
        filtered.filename = Some("alpha".to_owned());
        filtered.extension = Some(".PDF".to_owned());
        filtered.event_type = Some("created".to_owned());
        filtered.folder_id = Some(folder_id);
        filtered.presence = Some(crate::models::PresenceFilter::Present);
        filtered.date_from = Some("2000-01-01T00:00:00.000Z".to_owned());
        filtered.date_to = Some("2100-01-01T00:00:00.000Z".to_owned());
        let result = database
            .query_timeline_page(&filtered)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].file.name, "alpha.pdf");
    }

    #[test]
    fn file_disappearing_during_a_partial_scan_preserves_the_complete_snapshot() {
        let (_directory, database, root, folder_id, first_run_id) = setup();
        let stable_path = root.join("stable.txt");
        let disappearing_path = root.join("disappearing.txt");
        fs::write(&stable_path, b"stable").unwrap_or_else(|error| panic!("{error}"));
        fs::write(&disappearing_path, b"present").unwrap_or_else(|error| panic!("{error}"));
        let first = traverse(
            &database,
            first_run_id,
            folder_id,
            &root,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        database
            .complete_scan(
                first_run_id,
                folder_id,
                first.files_seen,
                first.warnings,
                first.errors,
            )
            .unwrap_or_else(|error| panic!("{error}"));
        let before = database
            .list_file_records(folder_id)
            .unwrap_or_else(|error| panic!("{error}"));
        let before_events = database
            .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("{error}"))
            .items
            .len();

        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("{error}"));
        let metadata = fs::symlink_metadata(&stable_path).unwrap_or_else(|error| panic!("{error}"));
        let staged =
            discovered_file(&stable_path, &metadata).unwrap_or_else(|error| panic!("{error}"));
        database
            .stage_scan_batch(run.scan_run_id, folder_id, &[staged], 1, 0, 0)
            .unwrap_or_else(|error| panic!("{error}"));
        fs::remove_file(&disappearing_path).unwrap_or_else(|error| panic!("{error}"));
        database
            .fail_scan(
                run.scan_run_id,
                TaskStatus::Failed,
                "missing_or_moved",
                1,
                0,
                1,
            )
            .unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(
            database.list_file_records(folder_id).unwrap_or_default(),
            before
        );
        assert_eq!(
            database
                .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
                .unwrap_or_else(|error| panic!("{error}"))
                .items
                .len(),
            before_events
        );
    }
}
