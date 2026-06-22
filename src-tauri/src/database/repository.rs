use std::sync::MutexGuard;

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, OptionalExtension, params, types::Type};

use crate::{
    errors::ChronicleError,
    models::{
        AvailabilityStatus, FileEvent, FileRecord, IndexedFolder, ScanRun, ScanTaskSnapshot,
        TaskStatus, TimelinePage,
    },
    scanner::DiscoveredFile,
};

use super::Database;

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn availability_reason(status: AvailabilityStatus) -> &'static str {
    match status {
        AvailabilityStatus::Available => "none",
        AvailabilityStatus::MissingOrMoved => "missing_or_moved",
        AvailabilityStatus::PermissionDenied => "permission_denied",
        AvailabilityStatus::Inaccessible => "inaccessible",
    }
}

fn legacy_availability(status: AvailabilityStatus) -> &'static str {
    match status {
        AvailabilityStatus::Available => "available",
        AvailabilityStatus::MissingOrMoved => "missing",
        AvailabilityStatus::PermissionDenied | AvailabilityStatus::Inaccessible => "unavailable",
    }
}

fn decode_availability(value: &str) -> Result<AvailabilityStatus, rusqlite::Error> {
    match value {
        "none" => Ok(AvailabilityStatus::Available),
        "missing_or_moved" => Ok(AvailabilityStatus::MissingOrMoved),
        "permission_denied" => Ok(AvailabilityStatus::PermissionDenied),
        "inaccessible" => Ok(AvailabilityStatus::Inaccessible),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown availability reason: {unexpected}").into(),
        )),
    }
}

fn decode_task_status(value: &str) -> Result<TaskStatus, rusqlite::Error> {
    match value {
        "running" => Ok(TaskStatus::Running),
        "completed" => Ok(TaskStatus::Completed),
        "failed" => Ok(TaskStatus::Failed),
        "cancelled" => Ok(TaskStatus::Cancelled),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown scan status: {unexpected}").into(),
        )),
    }
}

fn decode_nonnegative(value: i64, column: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
    })
}

fn map_folder(row: &rusqlite::Row<'_>) -> Result<IndexedFolder, rusqlite::Error> {
    let reason: String = row.get(6)?;
    Ok(IndexedFolder {
        id: row.get(0)?,
        normalized_path: row.get(1)?,
        display_name: row.get(2)?,
        added_at: row.get(3)?,
        last_successful_scan_at: row.get(4)?,
        monitoring_enabled: row.get::<_, i64>(5)? != 0,
        availability_status: decode_availability(&reason)?,
        last_checked_at: row.get(7)?,
    })
}

impl Database {
    fn connection(&self) -> Result<MutexGuard<'_, Connection>, ChronicleError> {
        self.connection
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)
    }

    pub fn schema_version(&self) -> Result<u32, ChronicleError> {
        let connection = self.connection()?;
        Ok(connection.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))?)
    }

    pub fn list_indexed_folders(&self) -> Result<Vec<IndexedFolder>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, normalized_path, display_name, added_at, last_successful_scan_at,
                    monitoring_enabled, availability_reason, last_checked_at
             FROM indexed_folders
             ORDER BY added_at ASC, id ASC",
        )?;
        let rows = statement.query_map([], map_folder)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_indexed_folder(&self, folder_id: i64) -> Result<IndexedFolder, ChronicleError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, normalized_path, display_name, added_at, last_successful_scan_at,
                        monitoring_enabled, availability_reason, last_checked_at
                 FROM indexed_folders WHERE id = ?1",
                [folder_id],
                map_folder,
            )
            .optional()?
            .ok_or(ChronicleError::FolderNotFound)
    }

    pub fn insert_indexed_folder(
        &self,
        normalized_path: &str,
        normalized_path_key: &str,
        display_name: &str,
    ) -> Result<IndexedFolder, ChronicleError> {
        let now = now_rfc3339();
        let connection = self.connection()?;
        let result = connection.execute(
            "INSERT INTO indexed_folders (
                normalized_path, normalized_path_key, display_name, added_at,
                monitoring_enabled, availability_status, availability_reason, last_checked_at
             ) VALUES (?1, ?2, ?3, ?4, 0, 'available', 'none', ?4)",
            params![normalized_path, normalized_path_key, display_name, now],
        );
        match result {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(ref code, _)) if code.extended_code == 2067 => {
                return Err(ChronicleError::DuplicateFolder);
            }
            Err(error) => return Err(error.into()),
        }
        let id = connection.last_insert_rowid();
        drop(connection);
        self.get_indexed_folder(id)
    }

    pub fn update_folder_availability(
        &self,
        folder_id: i64,
        status: AvailabilityStatus,
    ) -> Result<(), ChronicleError> {
        let changed = self.connection()?.execute(
            "UPDATE indexed_folders
             SET availability_status = ?2, availability_reason = ?3, last_checked_at = ?4
             WHERE id = ?1",
            params![
                folder_id,
                legacy_availability(status),
                availability_reason(status),
                now_rfc3339()
            ],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FolderNotFound);
        }
        Ok(())
    }

    pub fn remove_indexed_folder(&self, folder_id: i64) -> Result<(), ChronicleError> {
        let connection = self.connection()?;
        let running: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM scan_runs WHERE indexed_folder_id = ?1 AND status = 'running')",
            [folder_id],
            |row| row.get(0),
        )?;
        if running {
            return Err(ChronicleError::ScanAlreadyRunning);
        }
        let changed =
            connection.execute("DELETE FROM indexed_folders WHERE id = ?1", [folder_id])?;
        if changed == 0 {
            return Err(ChronicleError::FolderNotFound);
        }
        Ok(())
    }

    pub fn create_scan_run(&self, folder_id: i64) -> Result<ScanTaskSnapshot, ChronicleError> {
        let now = now_rfc3339();
        let connection = self.connection()?;
        let already_running: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM scan_runs WHERE indexed_folder_id = ?1 AND status = 'running')",
            [folder_id],
            |row| row.get(0),
        )?;
        if already_running {
            return Err(ChronicleError::ScanAlreadyRunning);
        }
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM indexed_folders WHERE id = ?1)",
            [folder_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(ChronicleError::FolderNotFound);
        }
        connection.execute(
            "INSERT INTO scan_runs (indexed_folder_id, started_at, status)
             VALUES (?1, ?2, 'running')",
            params![folder_id, now],
        )?;
        Ok(ScanTaskSnapshot {
            scan_run_id: connection.last_insert_rowid(),
            indexed_folder_id: folder_id,
            status: TaskStatus::Running,
            files_seen: 0,
            warning_count: 0,
            error_count: 0,
            cancellable: true,
        })
    }

    pub fn stage_scan_batch(
        &self,
        scan_run_id: i64,
        folder_id: i64,
        files: &[DiscoveredFile],
        files_seen: u64,
        warnings: u64,
        errors: u64,
    ) -> Result<(), ChronicleError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        {
            let mut statement = transaction.prepare(
                "INSERT OR REPLACE INTO scan_file_staging (
                    scan_run_id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    size_bytes, filesystem_created_at, filesystem_modified_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;
            for file in files {
                statement.execute(params![
                    scan_run_id,
                    folder_id,
                    file.normalized_path,
                    file.name,
                    file.parent_path,
                    file.extension,
                    i64::try_from(file.size_bytes).map_err(|_| ChronicleError::NumericOverflow)?,
                    file.filesystem_created_at,
                    file.filesystem_modified_at,
                ])?;
            }
        }
        transaction.execute(
            "UPDATE scan_runs SET files_seen = ?2, warning_count = ?3, error_count = ?4
             WHERE id = ?1 AND status = 'running'",
            params![
                scan_run_id,
                i64::try_from(files_seen).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(warnings).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(errors).map_err(|_| ChronicleError::NumericOverflow)?,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn complete_scan(
        &self,
        scan_run_id: i64,
        folder_id: i64,
        files_seen: u64,
        warnings: u64,
        errors: u64,
    ) -> Result<(), ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO files (
                indexed_folder_id, normalized_path, name, parent_path, extension, size_bytes,
                filesystem_created_at, filesystem_modified_at, first_indexed_at, last_seen_at,
                is_present
             )
             SELECT indexed_folder_id, normalized_path, name, parent_path, extension, size_bytes,
                    filesystem_created_at, filesystem_modified_at, ?3, ?3, 1
             FROM scan_file_staging WHERE scan_run_id = ?1 AND indexed_folder_id = ?2
             ON CONFLICT(indexed_folder_id, normalized_path) DO UPDATE SET
                name = excluded.name,
                parent_path = excluded.parent_path,
                extension = excluded.extension,
                size_bytes = excluded.size_bytes,
                filesystem_created_at = excluded.filesystem_created_at,
                filesystem_modified_at = excluded.filesystem_modified_at,
                last_seen_at = excluded.last_seen_at,
                is_present = 1",
            params![scan_run_id, folder_id, now],
        )?;
        transaction.execute(
            "DELETE FROM files
             WHERE indexed_folder_id = ?1
               AND normalized_path NOT IN (
                   SELECT normalized_path FROM scan_file_staging WHERE scan_run_id = ?2
               )",
            params![folder_id, scan_run_id],
        )?;
        transaction.execute(
            "UPDATE indexed_folders
             SET last_successful_scan_at = ?2, availability_status = 'available',
                 availability_reason = 'none', last_checked_at = ?2
             WHERE id = ?1",
            params![folder_id, now],
        )?;
        transaction.execute(
            "UPDATE scan_runs
             SET completed_at = ?2, status = 'completed', files_seen = ?3,
                 warning_count = ?4, error_count = ?5, failure_kind = NULL
             WHERE id = ?1 AND status = 'running'",
            params![
                scan_run_id,
                now,
                i64::try_from(files_seen).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(warnings).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(errors).map_err(|_| ChronicleError::NumericOverflow)?,
            ],
        )?;
        transaction.execute(
            "DELETE FROM scan_file_staging WHERE scan_run_id = ?1",
            [scan_run_id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn fail_scan(
        &self,
        scan_run_id: i64,
        status: TaskStatus,
        failure_kind: &str,
        files_seen: u64,
        warnings: u64,
        errors: u64,
    ) -> Result<(), ChronicleError> {
        let status_value = if status == TaskStatus::Cancelled {
            "cancelled"
        } else {
            "failed"
        };
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE scan_runs
             SET completed_at = ?2, status = ?3, files_seen = ?4,
                 warning_count = ?5, error_count = ?6, failure_kind = ?7
             WHERE id = ?1 AND status = 'running'",
            params![
                scan_run_id,
                now_rfc3339(),
                status_value,
                i64::try_from(files_seen).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(warnings).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(errors).map_err(|_| ChronicleError::NumericOverflow)?,
                failure_kind
            ],
        )?;
        transaction.execute(
            "DELETE FROM scan_file_staging WHERE scan_run_id = ?1",
            [scan_run_id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn scan_task_snapshot(&self, scan_run_id: i64) -> Result<ScanTaskSnapshot, ChronicleError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT indexed_folder_id, status, files_seen, warning_count, error_count
                 FROM scan_runs WHERE id = ?1",
                [scan_run_id],
                |row| {
                    let raw_status: String = row.get(1)?;
                    let status = decode_task_status(&raw_status)?;
                    Ok(ScanTaskSnapshot {
                        scan_run_id,
                        indexed_folder_id: row.get(0)?,
                        status,
                        files_seen: decode_nonnegative(row.get(2)?, 2)?,
                        warning_count: decode_nonnegative(row.get(3)?, 3)?,
                        error_count: decode_nonnegative(row.get(4)?, 4)?,
                        cancellable: status == TaskStatus::Running,
                    })
                },
            )
            .optional()?
            .ok_or(ChronicleError::ScanNotFound)
    }

    pub fn recover_interrupted_scans(&self) -> Result<(), ChronicleError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE scan_runs SET status = 'failed', completed_at = ?1,
             error_count = error_count + 1, failure_kind = 'database'
             WHERE status = 'running'",
            [now_rfc3339()],
        )?;
        transaction.execute(
            "DELETE FROM scan_file_staging
             WHERE scan_run_id NOT IN (SELECT id FROM scan_runs WHERE status = 'running')",
            [],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn list_file_records(&self, folder_id: i64) -> Result<Vec<FileRecord>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    size_bytes, filesystem_created_at, filesystem_modified_at,
                    first_indexed_at, last_seen_at, is_present
             FROM files WHERE indexed_folder_id = ?1 ORDER BY normalized_path",
        )?;
        let rows = statement.query_map([folder_id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                indexed_folder_id: row.get(1)?,
                normalized_path: row.get(2)?,
                name: row.get(3)?,
                parent_path: row.get(4)?,
                extension: row.get(5)?,
                size_bytes: row.get(6)?,
                filesystem_created_at: row.get(7)?,
                filesystem_modified_at: row.get(8)?,
                first_indexed_at: row.get(9)?,
                last_seen_at: row.get(10)?,
                is_present: row.get::<_, i64>(11)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn latest_scan_run(&self, folder_id: i64) -> Result<Option<ScanRun>, ChronicleError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id, indexed_folder_id, started_at, completed_at, status, files_seen,
                    warning_count, error_count, failure_kind
             FROM scan_runs WHERE indexed_folder_id = ?1 ORDER BY id DESC LIMIT 1",
                [folder_id],
                |row| {
                    let raw_status: String = row.get(4)?;
                    Ok(ScanRun {
                        id: row.get(0)?,
                        indexed_folder_id: row.get(1)?,
                        started_at: row.get(2)?,
                        completed_at: row.get(3)?,
                        status: decode_task_status(&raw_status)?,
                        files_seen: row.get(5)?,
                        warning_count: row.get(6)?,
                        error_count: row.get(7)?,
                        failure_kind: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn query_timeline_page(
        &self,
        cursor: Option<i64>,
        page_size: u32,
    ) -> Result<TimelinePage, ChronicleError> {
        let page_size = page_size.clamp(1, 100) as usize;
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, file_id, indexed_folder_id, event_type, detected_at, filesystem_time,
                    old_path, new_path, confidence, event_source
             FROM file_events WHERE (?1 IS NULL OR id < ?1) ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![cursor, (page_size + 1) as i64], |row| {
            Ok(FileEvent {
                id: row.get(0)?,
                file_id: row.get(1)?,
                indexed_folder_id: row.get(2)?,
                event_type: row.get(3)?,
                detected_at: row.get(4)?,
                filesystem_time: row.get(5)?,
                old_path: row.get(6)?,
                new_path: row.get(7)?,
                confidence: row.get(8)?,
                event_source: row.get(9)?,
            })
        })?;
        let mut items = rows.collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > page_size;
        if has_more {
            items.truncate(page_size);
        }
        let next_cursor = has_more
            .then(|| items.last().map(|event| event.id))
            .flatten();
        Ok(TimelinePage {
            items,
            next_cursor,
            has_more,
        })
    }
}
