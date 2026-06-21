use std::sync::MutexGuard;

use rusqlite::{Connection, params, types::Type};

use crate::{
    errors::ChronicleError,
    models::{AvailabilityStatus, FileEvent, IndexedFolder, TimelinePage},
};

use super::Database;

fn availability_status(value: &str) -> Result<AvailabilityStatus, rusqlite::Error> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "missing" => Ok(AvailabilityStatus::Missing),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown availability status: {unexpected}").into(),
        )),
    }
}

impl Database {
    fn connection(&self) -> Result<MutexGuard<'_, Connection>, ChronicleError> {
        self.connection
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)
    }

    pub fn schema_version(&self) -> Result<u32, ChronicleError> {
        let connection = self.connection()?;
        let version =
            connection.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))?;
        Ok(version)
    }

    pub fn list_indexed_folders(&self) -> Result<Vec<IndexedFolder>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, normalized_path, display_name, added_at, last_successful_scan_at,
                    monitoring_enabled, availability_status
             FROM indexed_folders
             ORDER BY added_at ASC, id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            let raw_status: String = row.get(6)?;
            Ok(IndexedFolder {
                id: row.get(0)?,
                normalized_path: row.get(1)?,
                display_name: row.get(2)?,
                added_at: row.get(3)?,
                last_successful_scan_at: row.get(4)?,
                monitoring_enabled: row.get::<_, i64>(5)? != 0,
                availability_status: availability_status(&raw_status)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
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
             FROM file_events
             WHERE (?1 IS NULL OR id < ?1)
             ORDER BY id DESC
             LIMIT ?2",
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
