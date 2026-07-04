mod migrations;
mod repository;

use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;

use crate::errors::ChronicleError;

#[derive(Clone)]
pub struct Database {
    pub(crate) connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ChronicleError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, ChronicleError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(mut connection: Connection) -> Result<Self, ChronicleError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;
        migrations::apply(&mut connection)?;
        let database = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        database.recover_interrupted_scans()?;
        Ok(database)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use tempfile::TempDir;

    use super::Database;

    fn new_database() -> (TempDir, Database) {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should be created: {error}"));
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        (directory, database)
    }

    #[test]
    fn initial_migration_creates_all_required_tables() {
        let (_directory, database) = new_database();
        let connection = database
            .connection
            .lock()
            .unwrap_or_else(|error| panic!("database mutex should not be poisoned: {error}"));
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
            .unwrap_or_else(|error| panic!("schema query should prepare: {error}"));
        let names = statement
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap_or_else(|error| panic!("schema query should execute: {error}"))
            .collect::<Result<BTreeSet<_>, _>>()
            .unwrap_or_else(|error| panic!("table names should decode: {error}"));

        for expected in [
            "indexed_folders",
            "files",
            "file_events",
            "scan_runs",
            "scan_file_staging",
            "file_path_history",
            "app_settings",
            "version_families",
            "version_family_members",
            "version_family_suggestions",
            "projects",
            "project_members",
            "project_suggestions",
            "activity_sessions",
            "activity_session_events",
            "activity_session_files",
        ] {
            assert!(names.contains(expected), "missing table: {expected}");
        }
        drop(statement);
        drop(connection);
        assert_eq!(database.schema_version().unwrap_or_default(), 7);
    }

    #[test]
    fn new_database_lists_no_indexed_folders() {
        let (_directory, database) = new_database();
        let folders = database
            .list_indexed_folders()
            .unwrap_or_else(|error| panic!("folder query should succeed: {error}"));
        assert!(folders.is_empty());
    }

    #[test]
    fn new_database_returns_a_real_empty_timeline_page() {
        let (_directory, database) = new_database();
        let page = database
            .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("timeline query should succeed: {error}"));
        assert!(page.items.is_empty());
        assert_eq!(page.next_cursor, None);
        assert!(!page.has_more);
    }

    #[test]
    fn reopening_marks_interrupted_scans_failed_without_a_partial_snapshot() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should be created: {error}"));
        let path = directory.path().join("chronicle.sqlite3");
        let database =
            Database::open(&path).unwrap_or_else(|error| panic!("database should open: {error}"));
        let root = directory.path().join("root");
        std::fs::create_dir(&root)
            .unwrap_or_else(|error| panic!("root should be created: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        let first_run = database
            .create_scan_run(folder.id)
            .unwrap_or_else(|error| panic!("first scan should start: {error}"));
        let file = crate::scanner::DiscoveredFile {
            normalized_path: root.join("stable.txt").to_string_lossy().into_owned(),
            name: "stable.txt".to_owned(),
            parent_path: root.to_string_lossy().into_owned(),
            extension: Some("txt".to_owned()),
            size_bytes: 6,
            filesystem_created_at: None,
            filesystem_modified_at: "2026-06-23T00:00:00.000Z".to_owned(),
            identity_key: None,
        };
        database
            .stage_scan_batch(first_run.scan_run_id, folder.id, &[file], 1, 0, 0)
            .unwrap_or_else(|error| panic!("first scan should stage: {error}"));
        database
            .complete_scan(first_run.scan_run_id, folder.id, 1, 0, 0)
            .unwrap_or_else(|error| panic!("first snapshot should publish: {error}"));
        let run = database
            .create_scan_run(folder.id)
            .unwrap_or_else(|error| panic!("interrupted scan should start: {error}"));
        drop(database);

        let reopened =
            Database::open(&path).unwrap_or_else(|error| panic!("database should reopen: {error}"));
        let snapshot = reopened
            .scan_task_snapshot(run.scan_run_id)
            .unwrap_or_else(|error| panic!("recovered run should load: {error}"));
        assert_eq!(snapshot.status, crate::models::TaskStatus::Failed);
        let files = reopened.list_file_records(folder.id).unwrap_or_default();
        assert_eq!(files.len(), 1);
        assert!(files[0].is_present);
        let events = reopened
            .query_timeline_page(&crate::models::TimelineRequest::first_page(50))
            .unwrap_or_else(|error| panic!("timeline should load: {error}"));
        assert_eq!(events.items.len(), 1);
        assert_eq!(events.items[0].event.event_type, "created");
    }
}
