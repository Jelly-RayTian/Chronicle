mod migrations;
mod repository;

use std::{fs, path::Path, sync::Mutex};

use rusqlite::Connection;

use crate::errors::ChronicleError;

pub struct Database {
    pub(crate) connection: Mutex<Connection>,
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
        Ok(Self {
            connection: Mutex::new(connection),
        })
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
            "app_settings",
        ] {
            assert!(names.contains(expected), "missing table: {expected}");
        }
        drop(statement);
        drop(connection);
        assert_eq!(database.schema_version().unwrap_or_default(), 1);
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
            .query_timeline_page(None, 50)
            .unwrap_or_else(|error| panic!("timeline query should succeed: {error}"));
        assert!(page.items.is_empty());
        assert_eq!(page.next_cursor, None);
        assert!(!page.has_more);
    }
}
