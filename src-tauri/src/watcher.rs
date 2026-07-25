use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{
    content_indexing,
    database::Database,
    errors::ChronicleError,
    models::{IndexedFolder, WatcherDesiredState, WatcherRuntimeState, WatcherStatus},
    platform::{
        NativePathNormalizer, PathNormalizer, classify_io_error, path_to_string, probe_directory,
    },
    scanner::{DiscoveredFile, discovered_file, start_scan},
    tasks::ScanTaskManager,
};

const DEFAULT_COALESCING_WINDOW_MS: u64 = 750;
const MAX_PENDING_EVENTS: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherObservation {
    Present(DiscoveredFile),
    Missing { normalized_path: String },
}

struct RuntimeControl {
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    wake_sender: mpsc::Sender<notify::Result<Event>>,
    join: Option<JoinHandle<()>>,
}

#[derive(Clone, Default)]
pub struct WatcherManager {
    runtimes: Arc<Mutex<HashMap<i64, RuntimeControl>>>,
}

impl Drop for WatcherManager {
    fn drop(&mut self) {
        let _ = self.shutdown_all();
    }
}

impl WatcherManager {
    pub fn enable_folder(
        &self,
        database: Database,
        folder_id: i64,
        coalescing_window_ms: Option<u64>,
    ) -> Result<WatcherStatus, ChronicleError> {
        let coalescing = coalescing_window_ms.unwrap_or(DEFAULT_COALESCING_WINDOW_MS);
        database.set_monitoring_desired_state(
            folder_id,
            WatcherDesiredState::Enabled,
            WatcherRuntimeState::Running,
            Some(coalescing),
        )?;
        self.start_runtime(database.clone(), folder_id, false)?;
        database.watcher_status(folder_id)
    }

    pub fn disable_folder(
        &self,
        database: &Database,
        folder_id: i64,
    ) -> Result<WatcherStatus, ChronicleError> {
        self.stop_runtime(folder_id)?;
        database.set_monitoring_desired_state(
            folder_id,
            WatcherDesiredState::Disabled,
            WatcherRuntimeState::Stopped,
            None,
        )
    }

    pub fn pause_folder(
        &self,
        database: &Database,
        folder_id: i64,
    ) -> Result<WatcherStatus, ChronicleError> {
        let status = database.watcher_status(folder_id)?;
        if matches!(status.desired_state, WatcherDesiredState::Disabled) {
            return Err(ChronicleError::WatcherNotRunning);
        }
        if let Some(control) = self.runtime(folder_id)? {
            control.paused.store(true, Ordering::Release);
        }
        database.set_monitoring_desired_state(
            folder_id,
            WatcherDesiredState::Paused,
            WatcherRuntimeState::Paused,
            Some(status.coalescing_window_ms),
        )
    }

    pub fn resume_folder(
        &self,
        database: Database,
        folder_id: i64,
    ) -> Result<WatcherStatus, ChronicleError> {
        let status = database.watcher_status(folder_id)?;
        database.set_monitoring_desired_state(
            folder_id,
            WatcherDesiredState::Enabled,
            WatcherRuntimeState::Running,
            Some(status.coalescing_window_ms),
        )?;
        if let Some(control) = self.runtime(folder_id)? {
            control.paused.store(false, Ordering::Release);
        } else {
            self.start_runtime(database.clone(), folder_id, false)?;
        }
        database.watcher_status(folder_id)
    }

    pub fn status(
        &self,
        database: &Database,
        folder_id: i64,
    ) -> Result<WatcherStatus, ChronicleError> {
        database.watcher_status(folder_id)
    }

    pub fn start_enabled_folders(
        &self,
        database: Database,
        tasks: ScanTaskManager,
    ) -> Result<(), ChronicleError> {
        for folder in database.monitoring_enabled_folders()? {
            let status = database.watcher_status(folder.id)?;
            let paused = matches!(status.desired_state, WatcherDesiredState::Paused);
            let _ = start_scan(database.clone(), tasks.clone(), folder.id);
            let _ = self.start_runtime(database.clone(), folder.id, paused);
        }
        Ok(())
    }

    pub fn stop_runtime(&self, folder_id: i64) -> Result<(), ChronicleError> {
        let runtime = self
            .runtimes
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)?
            .remove(&folder_id);
        if let Some(mut runtime) = runtime {
            runtime.stop.store(true, Ordering::Release);
            let _ = runtime.wake_sender.send(Ok(Event::new(EventKind::Any)));
            if let Some(join) = runtime.join.take() {
                let _ = join.join();
            }
        }
        Ok(())
    }

    pub fn shutdown_all(&self) -> Result<(), ChronicleError> {
        let runtimes = {
            let mut guard = self
                .runtimes
                .lock()
                .map_err(|_error| ChronicleError::DatabaseState)?;
            guard
                .drain()
                .map(|(_folder_id, runtime)| runtime)
                .collect::<Vec<_>>()
        };
        for mut runtime in runtimes {
            runtime.stop.store(true, Ordering::Release);
            let _ = runtime.wake_sender.send(Ok(Event::new(EventKind::Any)));
            if let Some(join) = runtime.join.take() {
                let _ = join.join();
            }
        }
        Ok(())
    }

    fn runtime(&self, folder_id: i64) -> Result<Option<RuntimeControlView>, ChronicleError> {
        let guard = self
            .runtimes
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)?;
        Ok(guard.get(&folder_id).map(|runtime| RuntimeControlView {
            paused: Arc::clone(&runtime.paused),
        }))
    }

    fn start_runtime(
        &self,
        database: Database,
        folder_id: i64,
        paused: bool,
    ) -> Result<(), ChronicleError> {
        self.stop_runtime(folder_id)?;
        let folder = database.get_indexed_folder(folder_id)?;
        if probe_directory(Path::new(&folder.normalized_path))
            != crate::models::AvailabilityStatus::Available
        {
            database.set_watcher_runtime_state(folder_id, WatcherRuntimeState::Unavailable)?;
            return Err(ChronicleError::MissingOrMoved);
        }
        let status = database.watcher_status(folder_id)?;
        let stop = Arc::new(AtomicBool::new(false));
        let paused_token = Arc::new(AtomicBool::new(paused));
        let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
        let thread_stop = Arc::clone(&stop);
        let thread_paused = Arc::clone(&paused_token);
        let thread_sender = sender.clone();
        let join = thread::spawn(move || {
            run_watcher_thread(
                database,
                folder,
                status.coalescing_window_ms,
                thread_stop,
                thread_paused,
                receiver,
                thread_sender,
            );
        });
        self.runtimes
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)?
            .insert(
                folder_id,
                RuntimeControl {
                    stop,
                    paused: paused_token,
                    wake_sender: sender,
                    join: Some(join),
                },
            );
        Ok(())
    }
}

struct RuntimeControlView {
    paused: Arc<AtomicBool>,
}

fn run_watcher_thread(
    database: Database,
    folder: IndexedFolder,
    coalescing_window_ms: u64,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    receiver: mpsc::Receiver<notify::Result<Event>>,
    sender: mpsc::Sender<notify::Result<Event>>,
) {
    let root = PathBuf::from(&folder.normalized_path);
    let watcher_result = RecommendedWatcher::new(
        move |result| {
            let _ = sender.send(result);
        },
        Config::default(),
    )
    .and_then(|mut watcher| {
        watcher.watch(&root, RecursiveMode::Recursive)?;
        Ok(watcher)
    });
    let Ok(_watcher) = watcher_result else {
        let _ = database.mark_watcher_error(
            folder.id,
            WatcherRuntimeState::Error,
            "watcher_start",
            "native watcher could not start",
        );
        return;
    };

    let mut pending = BTreeMap::<String, PathBuf>::new();
    let mut seen_raw_paths = HashSet::<PathBuf>::new();
    let coalescing_window = Duration::from_millis(coalescing_window_ms.clamp(100, 60_000));
    let mut next_flush: Option<Instant> = None;

    while !stop.load(Ordering::Acquire) {
        if paused.load(Ordering::Acquire) {
            let _ = receiver.recv_timeout(Duration::from_millis(100));
            continue;
        }
        let timeout = next_flush
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
            .unwrap_or_else(|| Duration::from_millis(100));
        match receiver.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                if !is_relevant_event(&event.kind) {
                    continue;
                }
                for raw_path in event.paths {
                    match queue_raw_path(&root, raw_path, &mut seen_raw_paths, &mut pending) {
                        Ok(queued) => {
                            if queued {
                                next_flush = Some(Instant::now() + coalescing_window);
                            }
                        }
                        Err(ChronicleError::EventStorm) => {
                            pending.clear();
                            seen_raw_paths.clear();
                            let _ = database.mark_watcher_error(
                                folder.id,
                                WatcherRuntimeState::Error,
                                "event_storm",
                                "too many filesystem events arrived before coalescing",
                            );
                            return;
                        }
                        Err(_) => {
                            let _ = database.mark_watcher_error(
                                folder.id,
                                WatcherRuntimeState::Running,
                                "unauthorized_event_path",
                                "ignored a filesystem event outside the authorized root",
                            );
                        }
                    }
                }
            }
            Ok(Err(_error)) => {
                let _ = database.mark_watcher_error(
                    folder.id,
                    WatcherRuntimeState::Error,
                    "watcher_event",
                    "native watcher reported an event error",
                );
                return;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if next_flush.is_some_and(|deadline| Instant::now() >= deadline)
                    && !pending.is_empty()
                {
                    let paths = pending.values().cloned().collect::<Vec<_>>();
                    pending.clear();
                    seen_raw_paths.clear();
                    next_flush = None;
                    match process_debounced_paths(&database, folder.id, &root, &paths) {
                        Ok(_) => {}
                        Err(
                            ChronicleError::MissingOrMoved
                            | ChronicleError::PermissionDenied
                            | ChronicleError::Inaccessible,
                        ) => {
                            let _ = database
                                .update_folder_availability(folder.id, probe_directory(&root));
                            let _ = database.set_watcher_runtime_state(
                                folder.id,
                                WatcherRuntimeState::Unavailable,
                            );
                            return;
                        }
                        Err(_) => {
                            let _ = database.mark_watcher_error(
                                folder.id,
                                WatcherRuntimeState::Error,
                                "watcher_publish",
                                "watcher observations could not be published",
                            );
                            return;
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = database.mark_watcher_error(
                    folder.id,
                    WatcherRuntimeState::Error,
                    "watcher_disconnected",
                    "native watcher channel disconnected",
                );
                return;
            }
        }
    }
    let _ = database.set_watcher_runtime_state(folder.id, WatcherRuntimeState::Stopped);
}

fn is_relevant_event(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
}

fn normalize_raw_event_path(root: &Path, raw_path: &Path) -> Result<String, ChronicleError> {
    if raw_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ChronicleError::UnauthorizedEventPath);
    }
    let normalized_root = PathBuf::from(path_to_string(root)?);
    if let Ok(canonical) = fs::canonicalize(raw_path) {
        let normalized = PathBuf::from(path_to_string(&canonical)?);
        if !normalized.starts_with(&normalized_root) {
            return Err(ChronicleError::UnauthorizedEventPath);
        }
        return path_to_string(&normalized);
    }
    let absolute = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        root.join(raw_path)
    };
    let normalized = PathBuf::from(path_to_string(&absolute)?);
    if !normalized.starts_with(&normalized_root) {
        return Err(ChronicleError::UnauthorizedEventPath);
    }
    path_to_string(&normalized)
}

fn queue_raw_path(
    root: &Path,
    raw_path: PathBuf,
    seen_raw_paths: &mut HashSet<PathBuf>,
    pending: &mut BTreeMap<String, PathBuf>,
) -> Result<bool, ChronicleError> {
    if seen_raw_paths.contains(&raw_path) {
        return Ok(false);
    }
    if seen_raw_paths.len() >= MAX_PENDING_EVENTS {
        return Err(ChronicleError::EventStorm);
    }
    let normalized = normalize_raw_event_path(root, &raw_path)?;
    seen_raw_paths.insert(raw_path.clone());
    pending.insert(normalized, raw_path);
    Ok(true)
}

#[cfg(test)]
pub(crate) fn coalesce_paths_for_test(
    root: &Path,
    raw_paths: &[PathBuf],
) -> Result<Vec<PathBuf>, ChronicleError> {
    let mut seen_raw_paths = HashSet::<PathBuf>::new();
    let mut pending = BTreeMap::<String, PathBuf>::new();
    for raw_path in raw_paths {
        queue_raw_path(root, raw_path.clone(), &mut seen_raw_paths, &mut pending)?;
    }
    Ok(pending.into_values().collect())
}

fn is_temporary_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return true;
    };
    let lower = name.to_lowercase();
    name.starts_with("~$")
        || name.starts_with(".#")
        || lower.ends_with(".tmp")
        || lower.ends_with(".temp")
        || lower.ends_with(".swp")
        || lower.ends_with(".swx")
        || lower.ends_with(".part")
        || lower.ends_with(".crdownload")
        || lower.ends_with('~')
}

fn recheck_path(
    root: &Path,
    raw_path: &Path,
) -> Result<Option<WatcherObservation>, ChronicleError> {
    let normalized = normalize_raw_event_path(root, raw_path)?;
    let path = PathBuf::from(&normalized);
    if is_temporary_path(&path) {
        return Ok(None);
    }
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_symlink() || !file_type.is_file() {
                return Ok(None);
            }
            Ok(Some(WatcherObservation::Present(discovered_file(
                &path, &metadata,
            )?)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(Some(WatcherObservation::Missing {
                normalized_path: normalized,
            }))
        }
        Err(error) => Err(classify_io_error(error)),
    }
}

fn process_debounced_paths(
    database: &Database,
    folder_id: i64,
    root: &Path,
    raw_paths: &[PathBuf],
) -> Result<u64, ChronicleError> {
    if probe_directory(root) != crate::models::AvailabilityStatus::Available {
        return Err(ChronicleError::MissingOrMoved);
    }
    let normalizer = NativePathNormalizer;
    let canonical_root = normalizer.normalize(root)?;
    let canonical_root = PathBuf::from(canonical_root);
    let mut observations = Vec::new();
    for raw_path in raw_paths {
        if let Some(observation) = recheck_path(&canonical_root, raw_path)? {
            observations.push(observation);
        }
    }
    let result = database.publish_watcher_observations(folder_id, &observations);
    if result.is_ok() {
        let _ = content_indexing::sync_files_from_observations(database, folder_id, &observations);
    }
    result
}

#[cfg(test)]
pub fn process_debounced_paths_for_test(
    database: &Database,
    folder_id: i64,
    root: &Path,
    raw_paths: &[PathBuf],
) -> Result<u64, ChronicleError> {
    process_debounced_paths(database, folder_id, root, raw_paths)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use tempfile::TempDir;

    use crate::{
        database::Database,
        folders::register_folder,
        models::{TimelineRequest, WatcherDesiredState, WatcherRuntimeState},
        scanner,
    };

    use super::{
        coalesce_paths_for_test, normalize_raw_event_path, process_debounced_paths_for_test,
    };

    fn setup() -> (TempDir, Database, PathBuf, i64) {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        (directory, database, root, folder.id)
    }

    fn scan(database: &Database, root: &Path, folder_id: i64) {
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("scan should start: {error}"));
        let counts = scanner::traverse(
            database,
            run.scan_run_id,
            folder_id,
            root,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("scan should traverse: {error}"));
        database
            .complete_scan(
                run.scan_run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("scan should publish: {error}"));
    }

    #[test]
    fn raw_burst_coalescing_discards_duplicate_paths_before_publication() {
        let (_directory, _database, root, _folder_id) = setup();
        let first = root.join("first.txt");
        let second = root.join("second.txt");
        fs::write(&first, b"first").unwrap_or_else(|error| panic!("write first: {error}"));
        fs::write(&second, b"second").unwrap_or_else(|error| panic!("write second: {error}"));
        let raw_paths = vec![
            first.clone(),
            second.clone(),
            first.clone(),
            second.clone(),
            first,
        ];
        let coalesced = coalesce_paths_for_test(&root, &raw_paths)
            .unwrap_or_else(|error| panic!("burst should coalesce: {error}"));
        assert_eq!(coalesced.len(), 2);
    }

    #[test]
    fn monitoring_control_states_persist_without_touching_files() {
        let (_directory, database, _root, folder_id) = setup();
        let enabled = database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Enabled,
                WatcherRuntimeState::Running,
                Some(250),
            )
            .unwrap_or_else(|error| panic!("watcher should enable: {error}"));
        assert_eq!(enabled.desired_state, WatcherDesiredState::Enabled);
        assert_eq!(enabled.runtime_state, WatcherRuntimeState::Running);
        let paused = database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Paused,
                WatcherRuntimeState::Paused,
                Some(250),
            )
            .unwrap_or_else(|error| panic!("watcher should pause: {error}"));
        assert_eq!(paused.desired_state, WatcherDesiredState::Paused);
        let resumed = database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Enabled,
                WatcherRuntimeState::Running,
                Some(250),
            )
            .unwrap_or_else(|error| panic!("watcher should resume: {error}"));
        assert_eq!(resumed.desired_state, WatcherDesiredState::Enabled);
        let disabled = database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Disabled,
                WatcherRuntimeState::Stopped,
                None,
            )
            .unwrap_or_else(|error| panic!("watcher should disable: {error}"));
        assert_eq!(disabled.desired_state, WatcherDesiredState::Disabled);
        assert_eq!(disabled.runtime_state, WatcherRuntimeState::Stopped);
    }

    #[test]
    fn duplicate_modify_events_coalesce_into_one_timeline_event() {
        let (_directory, database, root, folder_id) = setup();
        let file = root.join("note.txt");
        fs::write(&file, b"before").unwrap_or_else(|error| panic!("{error}"));
        scan(&database, &root, folder_id);
        fs::write(&file, b"after and larger").unwrap_or_else(|error| panic!("{error}"));
        let recorded = process_debounced_paths_for_test(
            &database,
            folder_id,
            &root,
            &[file.clone(), file.clone(), file],
        )
        .unwrap_or_else(|error| panic!("watcher batch should publish: {error}"));
        assert_eq!(recorded, 1);
        let page = database
            .query_timeline_page(&TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(page.items[0].event.event_type, "modified");
        assert_eq!(
            page.items
                .iter()
                .filter(|item| item.event.event_source == "watcher")
                .count(),
            1
        );
    }

    #[test]
    fn create_then_modify_and_temporary_patterns_publish_meaningful_events() {
        let (_directory, database, root, folder_id) = setup();
        database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Enabled,
                WatcherRuntimeState::Running,
                Some(750),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        let temp = root.join("draft.tmp");
        fs::write(&temp, b"temp").unwrap_or_else(|error| panic!("{error}"));
        let final_file = root.join("draft.txt");
        fs::write(&final_file, b"first").unwrap_or_else(|error| panic!("{error}"));
        fs::write(&final_file, b"first plus save").unwrap_or_else(|error| panic!("{error}"));
        let recorded = process_debounced_paths_for_test(
            &database,
            folder_id,
            &root,
            &[temp, final_file.clone(), final_file],
        )
        .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(recorded, 1);
        let page = database
            .query_timeline_page(&TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].event.event_type, "created");
        assert_eq!(page.items[0].file.name, "draft.txt");
    }

    #[test]
    fn events_outside_authorized_root_are_rejected() {
        let (directory, _database, root, _folder_id) = setup();
        let outside = directory.path().join("outside.txt");
        fs::write(&outside, b"outside").unwrap_or_else(|error| panic!("{error}"));
        assert!(normalize_raw_event_path(&root, &outside).is_err());
    }

    #[test]
    fn folder_removal_while_watching_does_not_create_false_deletions() {
        let (_directory, database, root, folder_id) = setup();
        let file = root.join("stable.txt");
        fs::write(&file, b"stable").unwrap_or_else(|error| panic!("{error}"));
        scan(&database, &root, folder_id);
        let before = database
            .query_timeline_page(&TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("{error}"))
            .items
            .len();
        fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("{error}"));
        assert!(process_debounced_paths_for_test(&database, folder_id, &root, &[file]).is_err());
        assert_eq!(
            database
                .query_timeline_page(&TimelineRequest::first_page(50))
                .unwrap_or_else(|error| panic!("{error}"))
                .items
                .len(),
            before
        );
    }

    #[test]
    fn restart_reconciliation_after_missed_events_uses_complete_scan() {
        let (_directory, database, root, folder_id) = setup();
        database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Enabled,
                WatcherRuntimeState::Stopped,
                Some(750),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        fs::write(root.join("missed.txt"), b"missed while closed")
            .unwrap_or_else(|error| panic!("{error}"));
        scan(&database, &root, folder_id);
        let page = database
            .query_timeline_page(&TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].event.event_source, "reconciliation");
    }

    #[test]
    fn failed_watcher_transaction_rolls_back_the_batch() {
        let (_directory, database, root, folder_id) = setup();
        database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Enabled,
                WatcherRuntimeState::Running,
                Some(750),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        let impossible =
            crate::watcher::WatcherObservation::Present(crate::scanner::DiscoveredFile {
                normalized_path: root.join("huge.bin").to_string_lossy().into_owned(),
                name: "huge.bin".to_owned(),
                parent_path: root.to_string_lossy().into_owned(),
                extension: Some("bin".to_owned()),
                size_bytes: u64::MAX,
                filesystem_created_at: None,
                filesystem_modified_at: "2026-06-23T00:00:00.000Z".to_owned(),
                identity_key: None,
            });
        assert!(
            database
                .publish_watcher_observations(folder_id, &[impossible])
                .is_err()
        );
        assert!(
            database
                .query_timeline_page(&TimelineRequest::first_page(50))
                .unwrap_or_else(|error| panic!("{error}"))
                .items
                .is_empty()
        );
    }

    #[test]
    fn watcher_failure_and_event_storm_keep_timeline_unchanged() {
        let (_directory, database, _root, folder_id) = setup();
        database
            .set_monitoring_desired_state(
                folder_id,
                WatcherDesiredState::Enabled,
                WatcherRuntimeState::Running,
                Some(750),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        database
            .mark_watcher_error(
                folder_id,
                WatcherRuntimeState::Error,
                "event_storm",
                "too many filesystem events arrived before coalescing",
            )
            .unwrap_or_else(|error| panic!("{error}"));

        let status = database
            .watcher_status(folder_id)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(status.runtime_state, WatcherRuntimeState::Error);
        assert_eq!(status.last_error_kind.as_deref(), Some("event_storm"));
        assert_eq!(status.events_dropped, 1);
        assert!(
            database
                .query_timeline_page(&TimelineRequest::first_page(50))
                .unwrap_or_else(|error| panic!("{error}"))
                .items
                .is_empty()
        );
    }
}
