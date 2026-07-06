use std::{
    collections::{HashMap, HashSet},
    sync::MutexGuard,
};

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, OptionalExtension, params, types::Type};

use crate::{
    errors::ChronicleError,
    models::{
        ActivitySession, ActivitySessionDetail, ActivitySessionStatus, ActivitySessionSummary,
        AvailabilityStatus, ContentSearchResult, FileEvent, FileRecord, FileVersionCandidate,
        IndexedFolder, ListProjectsRequest, ListSessionsRequest, ListVersionFamiliesRequest,
        PresenceFilter, Project, ProjectDecision, ProjectDetail, ProjectMember,
        ProjectMemberWithFile, ProjectMembershipType, ProjectStatus, ProjectSuggestion,
        ProjectSummary, ScanRun, ScanTaskSnapshot, TaskStatus, TimelineItem, TimelinePage,
        TimelineRequest, VersionFamily, VersionFamilyDecision, VersionFamilyDetail,
        VersionFamilyMember, VersionFamilyMemberWithFile, VersionFamilyStatus,
        VersionFamilySuggestion, VersionFamilySummary, WatcherDesiredState, WatcherRuntimeState,
        WatcherStatus,
    },
    scanner::DiscoveredFile,
    watcher::WatcherObservation,
};

type SuggestedProjectGroup = (String, Vec<(i64, f64, String, String)>);
type SessionTuple = (String, String, Option<i64>, Vec<i64>, Vec<i64>);

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

fn decode_watcher_desired(value: &str) -> Result<WatcherDesiredState, rusqlite::Error> {
    match value {
        "disabled" => Ok(WatcherDesiredState::Disabled),
        "enabled" => Ok(WatcherDesiredState::Enabled),
        "paused" => Ok(WatcherDesiredState::Paused),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown watcher desired state: {unexpected}").into(),
        )),
    }
}

fn decode_watcher_runtime(value: &str) -> Result<WatcherRuntimeState, rusqlite::Error> {
    match value {
        "stopped" => Ok(WatcherRuntimeState::Stopped),
        "running" => Ok(WatcherRuntimeState::Running),
        "paused" => Ok(WatcherRuntimeState::Paused),
        "unavailable" => Ok(WatcherRuntimeState::Unavailable),
        "error" => Ok(WatcherRuntimeState::Error),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown watcher runtime state: {unexpected}").into(),
        )),
    }
}

fn decode_family_status(value: &str) -> Result<VersionFamilyStatus, rusqlite::Error> {
    match value {
        "suggested" => Ok(VersionFamilyStatus::Suggested),
        "confirmed" => Ok(VersionFamilyStatus::Confirmed),
        "rejected" => Ok(VersionFamilyStatus::Rejected),
        "superseded" => Ok(VersionFamilyStatus::Superseded),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown version family status: {unexpected}").into(),
        )),
    }
}

fn decode_family_decision(
    value: Option<&str>,
) -> Result<Option<VersionFamilyDecision>, rusqlite::Error> {
    match value {
        None | Some("") => Ok(None),
        Some("accepted") => Ok(Some(VersionFamilyDecision::Accepted)),
        Some("rejected") => Ok(Some(VersionFamilyDecision::Rejected)),
        Some("split") => Ok(Some(VersionFamilyDecision::Split)),
        Some("merged") => Ok(Some(VersionFamilyDecision::Merged)),
        Some(unexpected) => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown version family decision: {unexpected}").into(),
        )),
    }
}

fn decode_project_status(value: &str) -> Result<ProjectStatus, rusqlite::Error> {
    match value {
        "suggested" => Ok(ProjectStatus::Suggested),
        "active" => Ok(ProjectStatus::Active),
        "archived" => Ok(ProjectStatus::Archived),
        "rejected" => Ok(ProjectStatus::Rejected),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown project status: {unexpected}").into(),
        )),
    }
}

fn decode_project_decision(
    value: Option<&str>,
) -> Result<Option<ProjectDecision>, rusqlite::Error> {
    match value {
        None | Some("") => Ok(None),
        Some("accepted") => Ok(Some(ProjectDecision::Accepted)),
        Some("rejected") => Ok(Some(ProjectDecision::Rejected)),
        Some(unexpected) => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown project decision: {unexpected}").into(),
        )),
    }
}

fn decode_project_membership_type(value: &str) -> Result<ProjectMembershipType, rusqlite::Error> {
    match value {
        "manual" => Ok(ProjectMembershipType::Manual),
        "suggested" => Ok(ProjectMembershipType::Suggested),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown project membership type: {unexpected}").into(),
        )),
    }
}

fn decode_session_status(value: &str) -> Result<ActivitySessionStatus, rusqlite::Error> {
    match value {
        "auto" => Ok(ActivitySessionStatus::Auto),
        "edited" => Ok(ActivitySessionStatus::Edited),
        "accepted" => Ok(ActivitySessionStatus::Accepted),
        "rejected" => Ok(ActivitySessionStatus::Rejected),
        unexpected => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            format!("unknown activity session status: {unexpected}").into(),
        )),
    }
}

fn decode_nonnegative(value: i64, column: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
    })
}

fn rename_similarity_heuristic(new_path: &str, old_path: &str) -> f64 {
    let new_name = std::path::Path::new(new_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let old_name = std::path::Path::new(old_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let same_parent =
        std::path::Path::new(new_path).parent() == std::path::Path::new(old_path).parent();

    let name_similarity = if new_name.eq_ignore_ascii_case(old_name) {
        1.0_f64
    } else {
        let common_prefix = new_name
            .chars()
            .zip(old_name.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let max_len = new_name.len().max(old_name.len()).max(1) as f64;
        (common_prefix as f64) / max_len
    };

    if same_parent && (new_name == old_name || name_similarity > 0.7) {
        name_similarity.max(0.6)
    } else if !same_parent && name_similarity > 0.7 {
        name_similarity * 0.8
    } else if name_similarity > 0.4 {
        name_similarity * 0.6
    } else {
        0.0
    }
}

fn watcher_desired_value(value: WatcherDesiredState) -> &'static str {
    match value {
        WatcherDesiredState::Disabled => "disabled",
        WatcherDesiredState::Enabled => "enabled",
        WatcherDesiredState::Paused => "paused",
    }
}

fn watcher_runtime_value(value: WatcherRuntimeState) -> &'static str {
    match value {
        WatcherRuntimeState::Stopped => "stopped",
        WatcherRuntimeState::Running => "running",
        WatcherRuntimeState::Paused => "paused",
        WatcherRuntimeState::Unavailable => "unavailable",
        WatcherRuntimeState::Error => "error",
    }
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
        content_indexing_enabled: row.get::<_, i64>(8)? != 0,
        content_indexing_extensions: row.get(9)?,
        content_indexing_max_bytes: row.get(10)?,
        content_indexing_exclusion_patterns: row.get(11)?,
    })
}

fn map_watcher_status(row: &rusqlite::Row<'_>) -> Result<WatcherStatus, rusqlite::Error> {
    let desired: String = row.get(1)?;
    let runtime: String = row.get(2)?;
    Ok(WatcherStatus {
        folder_id: row.get(0)?,
        desired_state: decode_watcher_desired(&desired)?,
        runtime_state: decode_watcher_runtime(&runtime)?,
        coalescing_window_ms: decode_nonnegative(row.get(3)?, 3)?,
        last_started_at: row.get(4)?,
        last_stopped_at: row.get(5)?,
        last_event_at: row.get(6)?,
        last_error_at: row.get(7)?,
        last_error_kind: row.get(8)?,
        last_error_message: row.get(9)?,
        events_recorded: decode_nonnegative(row.get(10)?, 10)?,
        events_dropped: decode_nonnegative(row.get(11)?, 11)?,
    })
}

fn map_project(row: &rusqlite::Row<'_>) -> Result<Project, rusqlite::Error> {
    let status: String = row.get(3)?;
    let decision: Option<String> = row.get(4)?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        status: decode_project_status(&status)?,
        decision: decode_project_decision(decision.as_deref())?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_project_member(row: &rusqlite::Row<'_>) -> Result<ProjectMember, rusqlite::Error> {
    let membership_type: String = row.get(3)?;
    Ok(ProjectMember {
        id: row.get(0)?,
        project_id: row.get(1)?,
        file_id: row.get(2)?,
        membership_type: decode_project_membership_type(&membership_type)?,
        added_at: row.get(4)?,
    })
}

fn map_project_suggestion(row: &rusqlite::Row<'_>) -> Result<ProjectSuggestion, rusqlite::Error> {
    Ok(ProjectSuggestion {
        id: row.get(0)?,
        project_id: row.get(1)?,
        file_id: row.get(2)?,
        confidence: row.get(3)?,
        evidence: row.get(4)?,
        source: row.get(5)?,
        handled: row.get::<_, i64>(6)? != 0,
        suggested_at: row.get(7)?,
    })
}

fn map_activity_session(row: &rusqlite::Row<'_>) -> Result<ActivitySession, rusqlite::Error> {
    let status: String = row.get(6)?;
    Ok(ActivitySession {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        started_at: row.get(3)?,
        ended_at: row.get(4)?,
        event_summary: row.get(5)?,
        status: decode_session_status(&status)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
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
                    monitoring_enabled, availability_reason, last_checked_at,
                    content_indexing_enabled, content_indexing_extensions,
                    content_indexing_max_bytes, content_indexing_exclusion_patterns
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
                        monitoring_enabled, availability_reason, last_checked_at,
                        content_indexing_enabled, content_indexing_extensions,
                        content_indexing_max_bytes, content_indexing_exclusion_patterns
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
        connection.execute(
            "INSERT INTO watcher_states (indexed_folder_id, desired_state, runtime_state)
             VALUES (?1, 'disabled', 'stopped')",
            [id],
        )?;
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
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let running: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM scan_runs WHERE indexed_folder_id = ?1 AND status = 'running')",
            [folder_id],
            |row| row.get(0),
        )?;
        if running {
            return Err(ChronicleError::ScanAlreadyRunning);
        }
        transaction.execute(
            "DELETE FROM content_index_fts WHERE doc_id IN (SELECT id FROM files WHERE indexed_folder_id = ?1)",
            [folder_id],
        )?;
        transaction.execute(
            "DELETE FROM content_index_documents WHERE file_id IN (SELECT id FROM files WHERE indexed_folder_id = ?1)",
            [folder_id],
        )?;
        let changed =
            transaction.execute("DELETE FROM indexed_folders WHERE id = ?1", [folder_id])?;
        if changed == 0 {
            return Err(ChronicleError::FolderNotFound);
        }
        transaction.commit()?;
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
                    size_bytes, filesystem_created_at, filesystem_modified_at, identity_key
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
                    file.identity_key,
                ])?;
            }
        }
        let changed = transaction.execute(
            "UPDATE scan_runs SET files_seen = ?2, warning_count = ?3, error_count = ?4
             WHERE id = ?1 AND status = 'running'",
            params![
                scan_run_id,
                i64::try_from(files_seen).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(warnings).map_err(|_| ChronicleError::NumericOverflow)?,
                i64::try_from(errors).map_err(|_| ChronicleError::NumericOverflow)?,
            ],
        )?;
        if changed == 0 {
            return Err(ChronicleError::Cancelled);
        }
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
        let valid_run: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM scan_runs
             WHERE id = ?1 AND indexed_folder_id = ?2 AND status = 'running')",
            params![scan_run_id, folder_id],
            |row| row.get(0),
        )?;
        let staged_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM scan_file_staging
             WHERE scan_run_id = ?1 AND indexed_folder_id = ?2",
            params![scan_run_id, folder_id],
            |row| row.get(0),
        )?;
        if !valid_run
            || staged_count
                != i64::try_from(files_seen).map_err(|_| ChronicleError::NumericOverflow)?
        {
            return Err(ChronicleError::DatabaseState);
        }

        type RawChangeRow = (String, String, Option<String>, Option<i64>, Option<String>);

        let raw_changes: Vec<RawChangeRow> = {
            let mut statement = transaction.prepare(
                "SELECT s.normalized_path,
                        CASE WHEN f.id IS NULL OR f.is_present = 0 THEN 'created'
                             ELSE 'modified' END as event_type,
                        s.filesystem_modified_at,
                        f.id as existing_file_id,
                        COALESCE(
                            CASE WHEN f.id IS NULL OR f.is_present = 0 THEN s.identity_key ELSE NULL END,
                            f.identity_key
                        ) as effective_identity_key
                 FROM scan_file_staging s
                 LEFT JOIN files f ON f.indexed_folder_id = s.indexed_folder_id
                                   AND f.normalized_path = s.normalized_path
                 WHERE s.scan_run_id = ?1 AND s.indexed_folder_id = ?2
                   AND (f.id IS NULL OR f.is_present = 0
                        OR f.name IS NOT s.name
                        OR f.parent_path IS NOT s.parent_path
                        OR f.extension IS NOT s.extension
                        OR f.size_bytes IS NOT s.size_bytes
                        OR f.filesystem_created_at IS NOT s.filesystem_created_at
                        OR f.filesystem_modified_at IS NOT s.filesystem_modified_at)
                 UNION ALL
                 SELECT f.normalized_path, 'deleted', NULL, f.id, f.identity_key
                 FROM files f
                 LEFT JOIN scan_file_staging s ON s.scan_run_id = ?1
                                               AND s.indexed_folder_id = f.indexed_folder_id
                                               AND s.normalized_path = f.normalized_path
                 WHERE f.indexed_folder_id = ?2 AND f.is_present = 1
                   AND s.normalized_path IS NULL",
            )?;
            let rows = statement.query_map(params![scan_run_id, folder_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let mut deleted_entries: Vec<(&str, i64, Option<&str>)> = Vec::new();
        let mut created_entries: Vec<(&str, Option<&str>)> = Vec::new();
        let mut modified_entries: Vec<(&str, Option<&str>)> = Vec::new();

        for (path, event_type, fs_time, file_id, identity_key) in &raw_changes {
            match event_type.as_str() {
                "deleted" => {
                    if let Some(fid) = *file_id {
                        deleted_entries.push((path.as_str(), fid, identity_key.as_deref()));
                    }
                }
                "created" => {
                    created_entries.push((path.as_str(), identity_key.as_deref()));
                }
                "modified" => {
                    modified_entries.push((path.as_str(), fs_time.as_deref()));
                }
                _ => {}
            }
        }

        let mut matched: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut rename_events: Vec<(i64, String, String, String, f64)> = Vec::new();

        for (created_path, created_identity) in &created_entries {
            if matched.contains(*created_path) {
                continue;
            }
            for (deleted_path, deleted_file_id, deleted_identity) in &deleted_entries {
                if matched.contains(*deleted_path) {
                    continue;
                }
                let identity_match = created_identity
                    .filter(|c| !c.is_empty())
                    .zip(deleted_identity.filter(|d| !d.is_empty()))
                    .is_some_and(|(c, d)| c == d);

                let (event_type, confidence) = if identity_match {
                    let is_rename = std::path::Path::new(created_path).parent()
                        == std::path::Path::new(deleted_path).parent();
                    (if is_rename { "renamed" } else { "moved" }, 1.0_f64)
                } else {
                    let score = rename_similarity_heuristic(created_path, deleted_path);
                    if score >= 0.8 {
                        ("likely_renamed", 0.8_f64)
                    } else if score >= 0.5 {
                        ("possible_move", 0.4_f64)
                    } else {
                        continue;
                    }
                };
                rename_events.push((
                    *deleted_file_id,
                    (*created_path).to_owned(),
                    (*deleted_path).to_owned(),
                    event_type.to_owned(),
                    confidence,
                ));
                matched.insert((*created_path).to_owned());
                matched.insert((*deleted_path).to_owned());
                break;
            }
        }

        for (file_id, new_path, old_path, event_type, confidence) in &rename_events {
            let (name, parent_path) = {
                let mut s = transaction.prepare(
                    "SELECT name, parent_path FROM scan_file_staging
                     WHERE scan_run_id = ?1 AND normalized_path = ?2",
                )?;
                s.query_row(params![scan_run_id, new_path], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
            };
            transaction.execute(
                "UPDATE files SET normalized_path = ?2, name = ?3, parent_path = ?4,
                 last_seen_at = ?5, is_present = 1
                 WHERE id = ?1 AND indexed_folder_id = ?6",
                params![file_id, new_path, name, parent_path, now, folder_id],
            )?;
            transaction.execute(
                "DELETE FROM scan_file_staging
                 WHERE scan_run_id = ?1 AND normalized_path = ?2",
                params![scan_run_id, new_path],
            )?;
            transaction.execute(
                "INSERT INTO file_path_history (
                    file_id, old_path, new_path, valid_from, valid_until,
                    confidence, evidence, event_source, detected_at
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, 'reconciliation', 'reconciliation', ?4)",
                params![file_id, old_path, new_path, now, confidence],
            )?;
            transaction.execute(
                "INSERT INTO file_events (
                    file_id, indexed_folder_id, event_type, detected_at,
                    filesystem_time, old_path, new_path, confidence, event_source
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, 'reconciliation')",
                params![
                    file_id,
                    folder_id,
                    event_type.as_str(),
                    now,
                    old_path,
                    new_path,
                    confidence,
                ],
            )?;
        }

        transaction.execute(
            "INSERT INTO files (
                indexed_folder_id, normalized_path, name, parent_path, extension, size_bytes,
                filesystem_created_at, filesystem_modified_at, first_indexed_at, last_seen_at,
                is_present, identity_key
             )
             SELECT indexed_folder_id, normalized_path, name, parent_path, extension, size_bytes,
                    filesystem_created_at, filesystem_modified_at, ?3, ?3, 1, identity_key
             FROM scan_file_staging WHERE scan_run_id = ?1 AND indexed_folder_id = ?2
             ON CONFLICT(indexed_folder_id, normalized_path) DO UPDATE SET
                name = excluded.name,
                parent_path = excluded.parent_path,
                extension = excluded.extension,
                size_bytes = excluded.size_bytes,
                filesystem_created_at = excluded.filesystem_created_at,
                filesystem_modified_at = excluded.filesystem_modified_at,
                identity_key = excluded.identity_key,
                last_seen_at = excluded.last_seen_at,
                is_present = 1",
            params![scan_run_id, folder_id, now],
        )?;

        transaction.execute(
            "UPDATE files SET is_present = 0
             WHERE indexed_folder_id = ?1 AND is_present = 1
               AND NOT EXISTS (
                   SELECT 1 FROM scan_file_staging s
                   WHERE s.scan_run_id = ?2
                     AND s.indexed_folder_id = files.indexed_folder_id
                     AND s.normalized_path = files.normalized_path
               )",
            params![folder_id, scan_run_id],
        )?;

        for (path, event_type, fs_time, _file_id, _identity_key) in raw_changes.iter() {
            let path_str = path.as_str();
            let ev_str = event_type.as_str();
            let fs = fs_time.as_deref();

            if matched.contains(path_str) {
                continue;
            }
            if !(ev_str == "created" || ev_str == "modified" || ev_str == "deleted") {
                continue;
            }

            let file_id: i64 = transaction.query_row(
                "SELECT id FROM files WHERE indexed_folder_id = ?1 AND normalized_path = ?2",
                params![folder_id, path_str],
                |row| row.get(0),
            )?;
            let (old_path, new_path) = match ev_str {
                "created" => (None, Some(path_str)),
                "deleted" => (Some(path_str), None),
                _ => (Some(path_str), Some(path_str)),
            };
            transaction.execute(
                "INSERT INTO file_events (
                    file_id, indexed_folder_id, event_type, detected_at, filesystem_time,
                    old_path, new_path, confidence, event_source
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1.0, 'reconciliation')",
                params![file_id, folder_id, ev_str, now, fs, old_path, new_path],
            )?;
        }

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

    pub fn cancel_scan(&self, scan_run_id: i64) -> Result<(), ChronicleError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE scan_runs
             SET completed_at = ?2, status = 'cancelled', failure_kind = 'cancelled'
             WHERE id = ?1 AND status = 'running'",
            params![scan_run_id, now_rfc3339()],
        )?;
        if changed == 0 {
            return Err(ChronicleError::ScanNotFound);
        }
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

    pub fn list_all_file_records(&self) -> Result<Vec<FileRecord>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    size_bytes, filesystem_created_at, filesystem_modified_at,
                    first_indexed_at, last_seen_at, is_present
             FROM files ORDER BY normalized_path",
        )?;
        let rows = statement.query_map([], |row| {
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

    pub fn list_present_file_records(&self) -> Result<Vec<FileRecord>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    size_bytes, filesystem_created_at, filesystem_modified_at,
                    first_indexed_at, last_seen_at, is_present
             FROM files
             WHERE is_present = 1
             ORDER BY normalized_path",
        )?;
        let rows = statement.query_map([], |row| {
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

    pub fn list_confirmed_version_families(
        &self,
    ) -> Result<Vec<(i64, String, Vec<i64>)>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT vf.id, vf.display_name, vfm.file_id
             FROM version_families vf
             JOIN version_family_members vfm ON vfm.version_family_id = vf.id
             WHERE vf.status = 'confirmed'
             ORDER BY vf.id, vfm.sort_order",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        let mut grouped: HashMap<i64, (String, Vec<i64>)> = HashMap::new();
        for (id, name, file_id) in rows.collect::<Result<Vec<_>, _>>()? {
            grouped
                .entry(id)
                .or_insert_with(|| (name, Vec::new()))
                .1
                .push(file_id);
        }
        Ok(grouped
            .into_iter()
            .map(|(id, (name, file_ids))| (id, name, file_ids))
            .collect())
    }

    pub fn map_files_to_project_ids(
        &self,
        file_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<i64>>, ChronicleError> {
        if file_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let connection = self.connection()?;
        let placeholders = file_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT pm.file_id, pm.project_id
             FROM project_members pm
             JOIN projects p ON p.id = pm.project_id
             WHERE pm.file_id IN ({}) AND p.status = 'active'",
            placeholders
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(file_ids.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut mapping: HashMap<i64, Vec<i64>> = HashMap::new();
        for (file_id, project_id) in rows.collect::<Result<Vec<_>, _>>()? {
            mapping.entry(file_id).or_default().push(project_id);
        }
        Ok(mapping)
    }

    pub fn list_recent_file_events(&self, limit: u32) -> Result<Vec<FileEvent>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, file_id, indexed_folder_id, event_type, detected_at,
                    filesystem_time, old_path, new_path, confidence, event_source,
                    user_confirmation, user_confirmed_at
             FROM file_events
             ORDER BY detected_at DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| {
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
                user_confirmation: row.get(10)?,
                user_confirmed_at: row.get(11)?,
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
        request: &TimelineRequest,
    ) -> Result<TimelinePage, ChronicleError> {
        let page_size = request.page_size.clamp(1, 100) as usize;
        let filename = request
            .filename
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        let filename_pattern = filename.map(|value| {
            format!(
                "%{}%",
                value
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            )
        });
        let extension = request
            .extension
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.trim_start_matches('.'));
        let presence = request.presence.map(|value| match value {
            PresenceFilter::Present => 1_i64,
            PresenceFilter::Deleted => 0_i64,
        });
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT e.id, e.file_id, e.indexed_folder_id, e.event_type, e.detected_at,
                    e.filesystem_time, e.old_path, e.new_path, e.confidence, e.event_source,
                    e.user_confirmation, e.user_confirmed_at,
                    f.id, f.indexed_folder_id, f.normalized_path, f.name, f.parent_path,
                    f.extension, f.size_bytes, f.filesystem_created_at, f.filesystem_modified_at,
                    f.first_indexed_at, f.last_seen_at, f.is_present,
                    d.display_name, d.normalized_path
             FROM file_events e
             JOIN files f ON f.id = e.file_id
             JOIN indexed_folders d ON d.id = e.indexed_folder_id
             WHERE (?1 IS NULL OR e.id < ?1)
               AND (?3 IS NULL OR f.name LIKE ?3 ESCAPE '\\')
               AND (?4 IS NULL OR lower(COALESCE(f.extension, '')) = lower(?4))
               AND (?5 IS NULL OR e.event_type = ?5)
               AND (?6 IS NULL OR e.indexed_folder_id = ?6)
               AND (?7 IS NULL OR e.detected_at >= ?7)
               AND (?8 IS NULL OR e.detected_at < ?8)
               AND (?9 IS NULL OR f.is_present = ?9)
             ORDER BY e.id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(
            params![
                request.cursor,
                (page_size + 1) as i64,
                filename_pattern,
                extension,
                request.event_type,
                request.folder_id,
                request.date_from,
                request.date_to,
                presence
            ],
            |row| {
                Ok(TimelineItem {
                    event: FileEvent {
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
                        user_confirmation: row.get(10)?,
                        user_confirmed_at: row.get(11)?,
                    },
                    file: FileRecord {
                        id: row.get(12)?,
                        indexed_folder_id: row.get(13)?,
                        normalized_path: row.get(14)?,
                        name: row.get(15)?,
                        parent_path: row.get(16)?,
                        extension: row.get(17)?,
                        size_bytes: row.get(18)?,
                        filesystem_created_at: row.get(19)?,
                        filesystem_modified_at: row.get(20)?,
                        first_indexed_at: row.get(21)?,
                        last_seen_at: row.get(22)?,
                        is_present: row.get::<_, i64>(23)? != 0,
                    },
                    folder_name: row.get(24)?,
                    folder_path: row.get(25)?,
                })
            },
        )?;
        let mut items = rows.collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > page_size;
        if has_more {
            items.truncate(page_size);
        }
        let next_cursor = has_more
            .then(|| items.last().map(|item| item.event.id))
            .flatten();
        Ok(TimelinePage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub fn file_event_history(
        &self,
        file_id: i64,
        limit: u32,
    ) -> Result<Vec<FileEvent>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, file_id, indexed_folder_id, event_type, detected_at, filesystem_time,
                    old_path, new_path, confidence, event_source,
                    user_confirmation, user_confirmed_at
             FROM file_events WHERE file_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows =
            statement.query_map(params![file_id, i64::from(limit.clamp(1, 200))], |row| {
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
                    user_confirmation: row.get(10)?,
                    user_confirmed_at: row.get(11)?,
                })
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn scan_history(&self, folder_id: i64, limit: u32) -> Result<Vec<ScanRun>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, started_at, completed_at, status, files_seen,
                    warning_count, error_count, failure_kind
             FROM scan_runs WHERE indexed_folder_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows =
            statement.query_map(params![folder_id, i64::from(limit.clamp(1, 100))], |row| {
                let raw: String = row.get(4)?;
                Ok(ScanRun {
                    id: row.get(0)?,
                    indexed_folder_id: row.get(1)?,
                    started_at: row.get(2)?,
                    completed_at: row.get(3)?,
                    status: decode_task_status(&raw)?,
                    files_seen: row.get(5)?,
                    warning_count: row.get(6)?,
                    error_count: row.get(7)?,
                    failure_kind: row.get(8)?,
                })
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn file_path_history(
        &self,
        file_id: i64,
        limit: u32,
    ) -> Result<Vec<crate::models::PathHistoryItem>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, file_id, old_path, new_path, valid_from, valid_until,
                    confidence, evidence, event_source, detected_at
             FROM file_path_history WHERE file_id = ?1 ORDER BY valid_from ASC LIMIT ?2",
        )?;
        let rows =
            statement.query_map(params![file_id, i64::from(limit.clamp(1, 100))], |row| {
                Ok(crate::models::PathHistoryItem {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    old_path: row.get(2)?,
                    new_path: row.get(3)?,
                    valid_from: row.get(4)?,
                    valid_until: row.get(5)?,
                    confidence: row.get(6)?,
                    evidence: row.get(7)?,
                    event_source: row.get(8)?,
                    detected_at: row.get(9)?,
                })
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn confirm_event(&self, event_id: i64) -> Result<(), ChronicleError> {
        let now = now_rfc3339();
        let changed = self.connection()?.execute(
            "UPDATE file_events
             SET user_confirmation = 'confirmed', user_confirmed_at = ?2
             WHERE id = ?1",
            params![event_id, now],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FileUnavailable);
        }
        Ok(())
    }

    pub fn reject_event(&self, event_id: i64) -> Result<(), ChronicleError> {
        let now = now_rfc3339();
        let changed = self.connection()?.execute(
            "UPDATE file_events
             SET user_confirmation = 'rejected', user_confirmed_at = ?2
             WHERE id = ?1",
            params![event_id, now],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FileUnavailable);
        }
        Ok(())
    }

    pub fn present_file_context(&self, file_id: i64) -> Result<(String, String), ChronicleError> {
        self.connection()?
            .query_row(
                "SELECT f.normalized_path, d.normalized_path
                 FROM files f JOIN indexed_folders d ON d.id = f.indexed_folder_id
                 WHERE f.id = ?1 AND f.is_present = 1",
                [file_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(ChronicleError::FileUnavailable)
    }

    pub fn list_watcher_statuses(&self) -> Result<Vec<WatcherStatus>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT indexed_folder_id, desired_state, runtime_state, coalescing_window_ms,
                    last_started_at, last_stopped_at, last_event_at, last_error_at,
                    last_error_kind, last_error_message, events_recorded, events_dropped
             FROM watcher_states ORDER BY indexed_folder_id ASC",
        )?;
        let rows = statement.query_map([], map_watcher_status)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn watcher_status(&self, folder_id: i64) -> Result<WatcherStatus, ChronicleError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT indexed_folder_id, desired_state, runtime_state, coalescing_window_ms,
                        last_started_at, last_stopped_at, last_event_at, last_error_at,
                        last_error_kind, last_error_message, events_recorded, events_dropped
                 FROM watcher_states WHERE indexed_folder_id = ?1",
                [folder_id],
                map_watcher_status,
            )
            .optional()?
            .ok_or(ChronicleError::FolderNotFound)
    }

    pub fn monitoring_enabled_folders(&self) -> Result<Vec<IndexedFolder>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT f.id, f.normalized_path, f.display_name, f.added_at,
                    f.last_successful_scan_at, f.monitoring_enabled,
                    f.availability_reason, f.last_checked_at
             FROM indexed_folders f
             JOIN watcher_states w ON w.indexed_folder_id = f.id
             WHERE f.monitoring_enabled = 1 AND w.desired_state IN ('enabled', 'paused')
             ORDER BY f.id ASC",
        )?;
        let rows = statement.query_map([], map_folder)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn set_monitoring_desired_state(
        &self,
        folder_id: i64,
        desired: WatcherDesiredState,
        runtime: WatcherRuntimeState,
        coalescing_window_ms: Option<u64>,
    ) -> Result<WatcherStatus, ChronicleError> {
        let now = now_rfc3339();
        let monitoring_enabled = i64::from(!matches!(desired, WatcherDesiredState::Disabled));
        let coalescing = coalescing_window_ms
            .map(|value| value.clamp(100, 60_000))
            .unwrap_or(750);
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE indexed_folders SET monitoring_enabled = ?2 WHERE id = ?1",
            params![folder_id, monitoring_enabled],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FolderNotFound);
        }
        transaction.execute(
            "INSERT INTO watcher_states (
                indexed_folder_id, desired_state, runtime_state, coalescing_window_ms,
                last_started_at, last_stopped_at, last_error_at, last_error_kind, last_error_message
             ) VALUES (?1, ?2, ?3, ?4,
                CASE WHEN ?3 IN ('running', 'paused') THEN ?5 ELSE NULL END,
                CASE WHEN ?3 = 'stopped' THEN ?5 ELSE NULL END,
                NULL, NULL, NULL)
             ON CONFLICT(indexed_folder_id) DO UPDATE SET
                desired_state = excluded.desired_state,
                runtime_state = excluded.runtime_state,
                coalescing_window_ms = excluded.coalescing_window_ms,
                last_started_at = CASE
                    WHEN excluded.runtime_state IN ('running', 'paused') THEN ?5
                    ELSE watcher_states.last_started_at
                END,
                last_stopped_at = CASE
                    WHEN excluded.runtime_state = 'stopped' THEN ?5
                    ELSE watcher_states.last_stopped_at
                END,
                last_error_at = NULL,
                last_error_kind = NULL,
                last_error_message = NULL",
            params![
                folder_id,
                watcher_desired_value(desired),
                watcher_runtime_value(runtime),
                i64::try_from(coalescing).map_err(|_| ChronicleError::NumericOverflow)?,
                now,
            ],
        )?;
        transaction.commit()?;
        drop(connection);
        self.watcher_status(folder_id)
    }

    pub fn set_watcher_runtime_state(
        &self,
        folder_id: i64,
        runtime: WatcherRuntimeState,
    ) -> Result<(), ChronicleError> {
        let changed = self.connection()?.execute(
            "UPDATE watcher_states
             SET runtime_state = ?2,
                 last_started_at = CASE WHEN ?2 IN ('running', 'paused') THEN ?3 ELSE last_started_at END,
                 last_stopped_at = CASE WHEN ?2 = 'stopped' THEN ?3 ELSE last_stopped_at END
             WHERE indexed_folder_id = ?1",
            params![folder_id, watcher_runtime_value(runtime), now_rfc3339()],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FolderNotFound);
        }
        Ok(())
    }

    pub fn mark_watcher_error(
        &self,
        folder_id: i64,
        runtime: WatcherRuntimeState,
        kind: &str,
        message: &str,
    ) -> Result<(), ChronicleError> {
        let changed = self.connection()?.execute(
            "UPDATE watcher_states
             SET runtime_state = ?2, last_error_at = ?3, last_error_kind = ?4,
                 last_error_message = ?5, events_dropped = events_dropped + 1
             WHERE indexed_folder_id = ?1",
            params![
                folder_id,
                watcher_runtime_value(runtime),
                now_rfc3339(),
                kind,
                message
            ],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FolderNotFound);
        }
        Ok(())
    }

    pub fn publish_watcher_observations(
        &self,
        folder_id: i64,
        observations: &[WatcherObservation],
    ) -> Result<u64, ChronicleError> {
        if observations.is_empty() {
            return Ok(0);
        }
        let now = now_rfc3339();
        let mut events_recorded = 0_u64;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let folder_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM indexed_folders WHERE id = ?1)",
            [folder_id],
            |row| row.get(0),
        )?;
        if !folder_exists {
            return Err(ChronicleError::FolderNotFound);
        }

        for observation in observations {
            match observation {
                WatcherObservation::Present(file) => {
                    let existing = transaction
                        .query_row(
                            "SELECT id, is_present, name, parent_path, extension, size_bytes,
                                    filesystem_created_at, filesystem_modified_at
                             FROM files WHERE indexed_folder_id = ?1 AND normalized_path = ?2",
                            params![folder_id, file.normalized_path],
                            |row| {
                                Ok((
                                    row.get::<_, i64>(0)?,
                                    row.get::<_, i64>(1)? != 0,
                                    row.get::<_, String>(2)?,
                                    row.get::<_, String>(3)?,
                                    row.get::<_, Option<String>>(4)?,
                                    row.get::<_, i64>(5)?,
                                    row.get::<_, Option<String>>(6)?,
                                    row.get::<_, String>(7)?,
                                ))
                            },
                        )
                        .optional()?;
                    let size_bytes = i64::try_from(file.size_bytes)
                        .map_err(|_| ChronicleError::NumericOverflow)?;
                    let event_type = existing.as_ref().and_then(
                        |(
                            _id,
                            is_present,
                            name,
                            parent_path,
                            extension,
                            existing_size,
                            created_at,
                            modified_at,
                        )| {
                            if !is_present {
                                Some("created")
                            } else if name != &file.name
                                || parent_path != &file.parent_path
                                || extension != &file.extension
                                || *existing_size != size_bytes
                                || created_at != &file.filesystem_created_at
                                || modified_at != &file.filesystem_modified_at
                            {
                                Some("modified")
                            } else {
                                None
                            }
                        },
                    );
                    let event_type = event_type.or(Some("created").filter(|_| existing.is_none()));
                    transaction.execute(
                        "INSERT INTO files (
                            indexed_folder_id, normalized_path, name, parent_path, extension,
                            size_bytes, filesystem_created_at, filesystem_modified_at,
                            first_indexed_at, last_seen_at, is_present, identity_key
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, 1, ?10)
                         ON CONFLICT(indexed_folder_id, normalized_path) DO UPDATE SET
                            name = excluded.name,
                            parent_path = excluded.parent_path,
                            extension = excluded.extension,
                            size_bytes = excluded.size_bytes,
                            filesystem_created_at = excluded.filesystem_created_at,
                            filesystem_modified_at = excluded.filesystem_modified_at,
                            identity_key = excluded.identity_key,
                            last_seen_at = excluded.last_seen_at,
                            is_present = 1",
                        params![
                            folder_id,
                            file.normalized_path,
                            file.name,
                            file.parent_path,
                            file.extension,
                            size_bytes,
                            file.filesystem_created_at,
                            file.filesystem_modified_at,
                            now,
                            file.identity_key,
                        ],
                    )?;
                    if let Some(event_type) = event_type {
                        let file_id: i64 = transaction.query_row(
                            "SELECT id FROM files WHERE indexed_folder_id = ?1 AND normalized_path = ?2",
                            params![folder_id, file.normalized_path],
                            |row| row.get(0),
                        )?;
                        let old_path = if event_type == "created" {
                            None
                        } else {
                            Some(file.normalized_path.as_str())
                        };
                        transaction.execute(
                            "INSERT INTO file_events (
                                file_id, indexed_folder_id, event_type, detected_at,
                                filesystem_time, old_path, new_path, confidence, event_source
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0.85, 'watcher')",
                            params![
                                file_id,
                                folder_id,
                                event_type,
                                now,
                                file.filesystem_modified_at,
                                old_path,
                                file.normalized_path
                            ],
                        )?;
                        events_recorded += 1;
                    }
                }
                WatcherObservation::Missing { normalized_path } => {
                    let existing = transaction
                        .query_row(
                            "SELECT id FROM files
                             WHERE indexed_folder_id = ?1 AND normalized_path = ?2 AND is_present = 1",
                            params![folder_id, normalized_path],
                            |row| row.get::<_, i64>(0),
                        )
                        .optional()?;
                    if let Some(file_id) = existing {
                        transaction.execute(
                            "UPDATE files SET is_present = 0, last_seen_at = ?3
                             WHERE indexed_folder_id = ?1 AND normalized_path = ?2",
                            params![folder_id, normalized_path, now],
                        )?;
                        transaction.execute(
                            "INSERT INTO file_events (
                                file_id, indexed_folder_id, event_type, detected_at,
                                filesystem_time, old_path, new_path, confidence, event_source
                             ) VALUES (?1, ?2, 'deleted', ?3, NULL, ?4, NULL, 0.85, 'watcher')",
                            params![file_id, folder_id, now, normalized_path],
                        )?;
                        events_recorded += 1;
                    }
                }
            }
        }
        if events_recorded > 0 {
            transaction.execute(
                "UPDATE watcher_states
                 SET last_event_at = ?2, events_recorded = events_recorded + ?3
                 WHERE indexed_folder_id = ?1",
                params![
                    folder_id,
                    now,
                    i64::try_from(events_recorded).map_err(|_| ChronicleError::NumericOverflow)?,
                ],
            )?;
        }
        transaction.commit()?;
        Ok(events_recorded)
    }

    pub fn files_for_version_analysis(
        &self,
        folder_id: Option<i64>,
    ) -> Result<Vec<FileVersionCandidate>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    filesystem_modified_at, first_indexed_at, identity_key, is_present
             FROM files
             WHERE (?1 IS NULL OR indexed_folder_id = ?1)
             ORDER BY id",
        )?;
        let rows = statement.query_map(params![folder_id], |row| {
            Ok(FileVersionCandidate {
                id: row.get(0)?,
                indexed_folder_id: row.get(1)?,
                normalized_path: row.get(2)?,
                name: row.get(3)?,
                parent_path: row.get(4)?,
                extension: row.get(5)?,
                filesystem_modified_at: row.get(6)?,
                first_indexed_at: row.get(7)?,
                identity_key: row.get(8)?,
                is_present: row.get::<_, i64>(9)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn existing_family_file_ids(&self) -> Result<HashSet<i64>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT file_id FROM version_family_members")?;
        let rows = statement.query_map([], |row| row.get::<_, i64>(0))?;
        rows.collect::<Result<HashSet<_>, _>>().map_err(Into::into)
    }

    pub fn path_history_for_candidates(
        &self,
        file_ids: &[i64],
    ) -> Result<Vec<(i64, String, f64)>, ChronicleError> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = file_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT file_id, old_path, confidence
             FROM file_path_history
             WHERE file_id IN ({})",
            placeholders
        );
        let connection = self.connection()?;
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(file_ids.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn renumber_family_members(
        transaction: &rusqlite::Transaction,
        family_id: i64,
    ) -> Result<(), ChronicleError> {
        let ids: Vec<i64> = {
            let mut statement = transaction.prepare(
                "SELECT id FROM version_family_members
                 WHERE version_family_id = ?1
                 ORDER BY sort_order, id",
            )?;
            let rows = statement.query_map([family_id], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        for (index, id) in ids.iter().enumerate() {
            transaction.execute(
                "UPDATE version_family_members SET sort_order = ?2 WHERE id = ?1",
                params![
                    id,
                    i64::try_from(index).map_err(|_| ChronicleError::NumericOverflow)?
                ],
            )?;
        }
        Ok(())
    }

    pub fn persist_version_family_suggestions(
        &self,
        families: &[(String, Vec<i64>, f64, String)],
    ) -> Result<(), ChronicleError> {
        if families.is_empty() {
            return Ok(());
        }
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        for (display_name, file_ids, confidence, evidence) in families {
            transaction.execute(
                "INSERT INTO version_families (
                    display_name, status, created_at, updated_at
                 ) VALUES (?1, 'suggested', ?2, ?2)",
                params![display_name, now],
            )?;
            let family_id = transaction.last_insert_rowid();
            for (sort_order, file_id) in file_ids.iter().enumerate() {
                transaction.execute(
                    "INSERT INTO version_family_members (
                        version_family_id, file_id, sort_order, added_at
                     ) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        family_id,
                        file_id,
                        i64::try_from(sort_order).map_err(|_| ChronicleError::NumericOverflow)?,
                        now
                    ],
                )?;
            }
            transaction.execute(
                "INSERT INTO version_family_suggestions (
                    version_family_id, confidence, evidence, detected_at, source
                 ) VALUES (?1, ?2, ?3, ?4, 'heuristic')",
                params![family_id, confidence, evidence, now],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn map_family(row: &rusqlite::Row<'_>) -> Result<VersionFamily, rusqlite::Error> {
        let status: String = row.get(2)?;
        let decision: Option<String> = row.get(3)?;
        Ok(VersionFamily {
            id: row.get(0)?,
            display_name: row.get(1)?,
            status: decode_family_status(&status)?,
            user_decision: decode_family_decision(decision.as_deref())?,
            user_decided_at: row.get(4)?,
            merged_into_family_id: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    pub fn list_version_families(
        &self,
        request: &ListVersionFamiliesRequest,
    ) -> Result<Vec<VersionFamilySummary>, ChronicleError> {
        let connection = self.connection()?;
        const SQL: &str =
            "SELECT vf.id, vf.display_name, vf.status, vf.user_decision, vf.user_decided_at,
                    vf.merged_into_family_id, vf.created_at, vf.updated_at,
                    COUNT(vfm.id) as member_count,
                    GROUP_CONCAT(vfm.file_id) as file_ids
             FROM version_families vf
             LEFT JOIN version_family_members vfm ON vfm.version_family_id = vf.id
             WHERE (?1 IS NULL OR vf.status = ?1)
               AND (?2 IS NULL OR EXISTS (
                   SELECT 1 FROM version_family_members m
                   JOIN files f ON f.id = m.file_id
                   WHERE m.version_family_id = vf.id AND f.indexed_folder_id = ?2
               ))
             GROUP BY vf.id
             ORDER BY vf.updated_at DESC, vf.id DESC";
        let mut statement = connection.prepare(SQL)?;
        let status_value = request.status.map(|status| status.as_str().to_owned());
        let rows = statement.query_map(params![status_value, request.folder_id], |row| {
            let family = Self::map_family(row)?;
            let member_count = row.get::<_, i64>(8)? as usize;
            let file_ids: Vec<i64> = row
                .get::<_, Option<String>>(9)?
                .unwrap_or_default()
                .split(',')
                .filter(|value| !value.is_empty())
                .filter_map(|value| value.parse().ok())
                .collect();
            Ok(VersionFamilySummary {
                family,
                member_count,
                file_ids,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_version_family(
        &self,
        family_id: i64,
    ) -> Result<VersionFamilyDetail, ChronicleError> {
        let connection = self.connection()?;
        let family = connection
            .query_row(
                "SELECT id, display_name, status, user_decision, user_decided_at,
                        merged_into_family_id, created_at, updated_at
                 FROM version_families WHERE id = ?1",
                [family_id],
                Self::map_family,
            )
            .optional()?
            .ok_or(ChronicleError::VersionFamilyNotFound)?;

        let mut statement = connection.prepare(
            "SELECT vfm.id, vfm.version_family_id, vfm.file_id, vfm.sort_order, vfm.added_at,
                    f.id, f.indexed_folder_id, f.normalized_path, f.name, f.parent_path,
                    f.extension, f.size_bytes, f.filesystem_created_at, f.filesystem_modified_at,
                    f.first_indexed_at, f.last_seen_at, f.is_present
             FROM version_family_members vfm
             JOIN files f ON f.id = vfm.file_id
             WHERE vfm.version_family_id = ?1
             ORDER BY vfm.sort_order, vfm.id",
        )?;
        let members = statement
            .query_map([family_id], |row| {
                Ok(VersionFamilyMemberWithFile {
                    member: VersionFamilyMember {
                        id: row.get(0)?,
                        version_family_id: row.get(1)?,
                        file_id: row.get(2)?,
                        sort_order: row.get(3)?,
                        added_at: row.get(4)?,
                    },
                    file: FileRecord {
                        id: row.get(5)?,
                        indexed_folder_id: row.get(6)?,
                        normalized_path: row.get(7)?,
                        name: row.get(8)?,
                        parent_path: row.get(9)?,
                        extension: row.get(10)?,
                        size_bytes: row.get(11)?,
                        filesystem_created_at: row.get(12)?,
                        filesystem_modified_at: row.get(13)?,
                        first_indexed_at: row.get(14)?,
                        last_seen_at: row.get(15)?,
                        is_present: row.get::<_, i64>(16)? != 0,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut suggestions_statement = connection.prepare(
            "SELECT id, version_family_id, confidence, evidence, detected_at, source
             FROM version_family_suggestions
             WHERE version_family_id = ?1
             ORDER BY detected_at DESC",
        )?;
        let suggestions = suggestions_statement
            .query_map([family_id], |row| {
                Ok(VersionFamilySuggestion {
                    id: row.get(0)?,
                    version_family_id: row.get(1)?,
                    confidence: row.get(2)?,
                    evidence: row.get(3)?,
                    detected_at: row.get(4)?,
                    source: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(VersionFamilyDetail {
            family,
            members,
            suggestions,
        })
    }

    pub fn version_family_suggestions(
        &self,
        family_id: i64,
    ) -> Result<Vec<VersionFamilySuggestion>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, version_family_id, confidence, evidence, detected_at, source
             FROM version_family_suggestions
             WHERE version_family_id = ?1
             ORDER BY detected_at DESC",
        )?;
        let rows = statement.query_map([family_id], |row| {
            Ok(VersionFamilySuggestion {
                id: row.get(0)?,
                version_family_id: row.get(1)?,
                confidence: row.get(2)?,
                evidence: row.get(3)?,
                detected_at: row.get(4)?,
                source: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update_version_family_decision(
        &self,
        family_id: i64,
        status: VersionFamilyStatus,
        decision: Option<VersionFamilyDecision>,
    ) -> Result<VersionFamily, ChronicleError> {
        let now = now_rfc3339();
        let changed = self.connection()?.execute(
            "UPDATE version_families
             SET status = ?2, user_decision = ?3, user_decided_at = ?4, updated_at = ?4
             WHERE id = ?1",
            params![
                family_id,
                status.as_str(),
                decision.map(VersionFamilyDecision::as_str),
                now
            ],
        )?;
        if changed == 0 {
            return Err(ChronicleError::VersionFamilyNotFound);
        }
        self.get_version_family(family_id)
            .map(|detail| detail.family)
    }

    pub fn rename_version_family(
        &self,
        family_id: i64,
        display_name: &str,
    ) -> Result<VersionFamily, ChronicleError> {
        let trimmed = display_name.trim();
        if trimmed.is_empty() || trimmed.len() > 200 {
            return Err(ChronicleError::PathEncoding);
        }
        let now = now_rfc3339();
        let changed = self.connection()?.execute(
            "UPDATE version_families
             SET display_name = ?2, updated_at = ?3
             WHERE id = ?1",
            params![family_id, trimmed, now],
        )?;
        if changed == 0 {
            return Err(ChronicleError::VersionFamilyNotFound);
        }
        self.get_version_family(family_id)
            .map(|detail| detail.family)
    }

    fn ensure_file_is_free_for_family(
        transaction: &rusqlite::Transaction,
        family_id: i64,
        file_id: i64,
    ) -> Result<(), ChronicleError> {
        let existing: Option<i64> = transaction
            .query_row(
                "SELECT version_family_id FROM version_family_members
                 WHERE file_id = ?1 AND version_family_id != ?2",
                params![file_id, family_id],
                |row| row.get(0),
            )
            .optional()?;
        if existing.is_some() {
            return Err(ChronicleError::DuplicateFamilyMember);
        }
        let file_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM files WHERE id = ?1)",
            [file_id],
            |row| row.get(0),
        )?;
        if !file_exists {
            return Err(ChronicleError::FileNotFound);
        }
        Ok(())
    }

    pub fn add_version_family_member(
        &self,
        family_id: i64,
        file_id: i64,
    ) -> Result<VersionFamilyDetail, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let family_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM version_families WHERE id = ?1)",
            [family_id],
            |row| row.get(0),
        )?;
        if !family_exists {
            return Err(ChronicleError::VersionFamilyNotFound);
        }
        let already_in_family: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM version_family_members
                 WHERE version_family_id = ?1 AND file_id = ?2)",
            params![family_id, file_id],
            |row| row.get(0),
        )?;
        if already_in_family {
            return Err(ChronicleError::DuplicateFamilyMember);
        }
        Self::ensure_file_is_free_for_family(&transaction, family_id, file_id)?;
        transaction.execute(
            "INSERT INTO version_family_members (
                version_family_id, file_id, sort_order, added_at
             ) VALUES (?1, ?2, (
                 SELECT COALESCE(MAX(sort_order), -1) + 1
                 FROM version_family_members
                 WHERE version_family_id = ?1
             ), ?3)",
            params![family_id, file_id, now],
        )?;
        Self::renumber_family_members(&transaction, family_id)?;
        transaction.execute(
            "UPDATE version_families SET updated_at = ?2 WHERE id = ?1",
            params![family_id, now],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_version_family(family_id)
    }

    pub fn remove_version_family_member(
        &self,
        family_id: i64,
        file_id: i64,
    ) -> Result<VersionFamilyDetail, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "DELETE FROM version_family_members
             WHERE version_family_id = ?1 AND file_id = ?2",
            params![family_id, file_id],
        )?;
        if changed == 0 {
            return Err(ChronicleError::FileNotFound);
        }
        Self::renumber_family_members(&transaction, family_id)?;
        let remaining: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM version_family_members WHERE version_family_id = ?1",
            [family_id],
            |row| row.get(0),
        )?;
        if remaining == 0 {
            transaction.execute(
                "UPDATE version_families
                 SET status = 'rejected', user_decision = 'rejected',
                     user_decided_at = ?2, updated_at = ?2
                 WHERE id = ?1",
                params![family_id, now],
            )?;
        } else {
            transaction.execute(
                "UPDATE version_families SET updated_at = ?2 WHERE id = ?1",
                params![family_id, now],
            )?;
        }
        transaction.commit()?;
        drop(connection);
        self.get_version_family(family_id)
    }

    pub fn split_version_family(
        &self,
        family_id: i64,
        file_ids: &[i64],
        display_name: Option<&str>,
    ) -> Result<VersionFamilyDetail, ChronicleError> {
        if file_ids.is_empty() {
            return Err(ChronicleError::FileNotFound);
        }
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;

        let original_name: String = transaction
            .query_row(
                "SELECT display_name FROM version_families WHERE id = ?1",
                [family_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(ChronicleError::VersionFamilyNotFound)?;

        let placeholders = file_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT file_id FROM version_family_members
             WHERE version_family_id = ?1 AND file_id IN ({})",
            placeholders
        );
        let found: HashSet<i64> = {
            let mut statement = transaction.prepare(&sql)?;
            let rows = statement.query_map(
                rusqlite::params_from_iter(std::iter::once(&family_id).chain(file_ids.iter())),
                |row| row.get::<_, i64>(0),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        }
        .into_iter()
        .collect();
        if found.len() != file_ids.len() {
            return Err(ChronicleError::FileNotFound);
        }

        let new_name = display_name
            .map(str::trim)
            .filter(|value| !value.is_empty() && value.len() <= 200)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{} (split)", original_name));

        transaction.execute(
            "INSERT INTO version_families (
                display_name, status, user_decision, user_decided_at, created_at, updated_at
             ) VALUES (?1, 'confirmed', 'accepted', ?2, ?2, ?2)",
            params![new_name, now],
        )?;
        let new_family_id = transaction.last_insert_rowid();

        for file_id in file_ids {
            transaction.execute(
                "DELETE FROM version_family_members
                 WHERE version_family_id = ?1 AND file_id = ?2",
                params![family_id, file_id],
            )?;
        }

        for (sort_order, file_id) in file_ids.iter().enumerate() {
            transaction.execute(
                "INSERT INTO version_family_members (
                    version_family_id, file_id, sort_order, added_at
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![
                    new_family_id,
                    file_id,
                    i64::try_from(sort_order).map_err(|_| ChronicleError::NumericOverflow)?,
                    now
                ],
            )?;
        }

        Self::renumber_family_members(&transaction, family_id)?;
        Self::renumber_family_members(&transaction, new_family_id)?;

        transaction.execute(
            "UPDATE version_families
             SET status = 'superseded', user_decision = 'split',
                 user_decided_at = ?2, updated_at = ?2
             WHERE id = ?1",
            params![family_id, now],
        )?;

        transaction.commit()?;
        drop(connection);
        self.get_version_family(new_family_id)
    }

    pub fn merge_version_families(
        &self,
        target_family_id: i64,
        source_family_ids: &[i64],
    ) -> Result<VersionFamilyDetail, ChronicleError> {
        if source_family_ids.is_empty() {
            return self.get_version_family(target_family_id);
        }
        if source_family_ids.contains(&target_family_id) {
            return Err(ChronicleError::VersionFamilyNotFound);
        }
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;

        let target_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM version_families WHERE id = ?1)",
            [target_family_id],
            |row| row.get(0),
        )?;
        if !target_exists {
            return Err(ChronicleError::VersionFamilyNotFound);
        }

        let placeholders = source_family_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT id FROM version_families WHERE id IN ({})",
            placeholders
        );
        let found: HashSet<i64> = {
            let mut statement = transaction.prepare(&sql)?;
            let rows = statement.query_map(
                rusqlite::params_from_iter(source_family_ids.iter()),
                |row| row.get::<_, i64>(0),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        }
        .into_iter()
        .collect();
        if found.len() != source_family_ids.len() {
            return Err(ChronicleError::VersionFamilyNotFound);
        }

        let existing_target_members: HashSet<i64> = {
            let mut statement = transaction.prepare(
                "SELECT file_id FROM version_family_members WHERE version_family_id = ?1",
            )?;
            let rows = statement.query_map([target_family_id], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        }
        .into_iter()
        .collect();

        let source_members_sql = format!(
            "SELECT version_family_id, file_id FROM version_family_members
             WHERE version_family_id IN ({})",
            placeholders
        );
        let source_members: Vec<(i64, i64)> = {
            let mut statement = transaction.prepare(&source_members_sql)?;
            let rows = statement.query_map(
                rusqlite::params_from_iter(source_family_ids.iter()),
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let mut next_order: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM version_family_members WHERE version_family_id = ?1",
            [target_family_id],
            |row| row.get(0),
        )?;

        for (source_family_id, file_id) in source_members {
            if existing_target_members.contains(&file_id) {
                transaction.execute(
                    "DELETE FROM version_family_members
                     WHERE version_family_id = ?1 AND file_id = ?2",
                    params![source_family_id, file_id],
                )?;
                continue;
            }
            transaction.execute(
                "DELETE FROM version_family_members
                 WHERE version_family_id = ?1 AND file_id = ?2",
                params![source_family_id, file_id],
            )?;
            transaction.execute(
                "INSERT INTO version_family_members (
                    version_family_id, file_id, sort_order, added_at
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![target_family_id, file_id, next_order, now],
            )?;
            next_order += 1;
        }

        for source_family_id in source_family_ids {
            transaction.execute(
                "UPDATE version_families
                 SET status = 'superseded', user_decision = 'merged',
                     merged_into_family_id = ?2, user_decided_at = ?3, updated_at = ?3
                 WHERE id = ?1",
                params![source_family_id, target_family_id, now],
            )?;
        }

        Self::renumber_family_members(&transaction, target_family_id)?;
        transaction.execute(
            "UPDATE version_families
             SET status = 'confirmed', user_decision = 'accepted',
                 user_decided_at = ?2, updated_at = ?2
             WHERE id = ?1 AND status != 'confirmed'",
            params![target_family_id, now],
        )?;

        transaction.commit()?;
        drop(connection);
        self.get_version_family(target_family_id)
    }

    pub fn create_project(
        &self,
        name: &str,
        description: &str,
        file_ids: &[i64],
    ) -> Result<ProjectDetail, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;

        transaction.execute(
            "INSERT INTO projects (name, description, status, decision, created_at, updated_at)
             VALUES (?1, ?2, 'active', 'accepted', ?3, ?3)",
            params![name, description, now],
        )?;
        let project_id = transaction.last_insert_rowid();

        for file_id in file_ids {
            transaction.execute(
                "INSERT INTO project_members (project_id, file_id, membership_type, added_at)
                 VALUES (?1, ?2, 'manual', ?3)",
                params![project_id, file_id, now],
            )?;
        }

        transaction.commit()?;
        drop(connection);
        self.get_project(project_id)
    }

    pub fn list_projects(
        &self,
        request: &ListProjectsRequest,
    ) -> Result<Vec<ProjectSummary>, ChronicleError> {
        let projects: Vec<Project> = {
            let connection = self.connection()?;
            let sql = if request.status.is_some() {
                "SELECT id, name, description, status, decision, created_at, updated_at
                 FROM projects WHERE status = ?1 ORDER BY updated_at DESC, id DESC"
            } else {
                "SELECT id, name, description, status, decision, created_at, updated_at
                 FROM projects ORDER BY updated_at DESC, id DESC"
            };
            let mut statement = connection.prepare(sql)?;
            let rows = if let Some(status) = request.status {
                statement.query_map([status.as_str()], map_project)?
            } else {
                statement.query_map([], map_project)?
            };
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let mut summaries = Vec::with_capacity(projects.len());
        for project in projects {
            let members = self.list_project_member_rows(project.id)?;
            summaries.push(ProjectSummary {
                member_count: members.len(),
                file_ids: members.iter().map(|member| member.file_id).collect(),
                project,
            });
        }
        Ok(summaries)
    }

    fn list_project_member_rows(
        &self,
        project_id: i64,
    ) -> Result<Vec<ProjectMember>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, project_id, file_id, membership_type, added_at
             FROM project_members WHERE project_id = ?1 ORDER BY id ASC",
        )?;
        let rows = statement.query_map([project_id], map_project_member)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_project(&self, project_id: i64) -> Result<ProjectDetail, ChronicleError> {
        let connection = self.connection()?;
        let project: Project = connection
            .query_row(
                "SELECT id, name, description, status, decision, created_at, updated_at
                 FROM projects WHERE id = ?1",
                [project_id],
                map_project,
            )
            .optional()?
            .ok_or(ChronicleError::ProjectNotFound)?;

        let members = {
            let mut statement = connection.prepare(
                "SELECT m.id, m.project_id, m.file_id, m.membership_type, m.added_at,
                        f.id, f.indexed_folder_id, f.normalized_path, f.name, f.parent_path,
                        f.extension, f.size_bytes, f.filesystem_created_at,
                        f.filesystem_modified_at, f.first_indexed_at, f.last_seen_at, f.is_present
                 FROM project_members m
                 JOIN files f ON f.id = m.file_id
                 WHERE m.project_id = ?1
                 ORDER BY m.id ASC",
            )?;
            let rows = statement.query_map([project_id], |row| {
                let member = ProjectMember {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    file_id: row.get(2)?,
                    membership_type: decode_project_membership_type(&row.get::<_, String>(3)?)?,
                    added_at: row.get(4)?,
                };
                let file = FileRecord {
                    id: row.get(5)?,
                    indexed_folder_id: row.get(6)?,
                    normalized_path: row.get(7)?,
                    name: row.get(8)?,
                    parent_path: row.get(9)?,
                    extension: row.get(10)?,
                    size_bytes: row.get(11)?,
                    filesystem_created_at: row.get(12)?,
                    filesystem_modified_at: row.get(13)?,
                    first_indexed_at: row.get(14)?,
                    last_seen_at: row.get(15)?,
                    is_present: row.get::<_, i64>(16)? != 0,
                };
                Ok(ProjectMemberWithFile { member, file })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let suggestions = {
            let mut statement = connection.prepare(
                "SELECT id, project_id, file_id, confidence, evidence, source, handled, suggested_at
                 FROM project_suggestions
                 WHERE project_id = ?1 AND handled = 0
                 ORDER BY confidence DESC, id ASC",
            )?;
            let rows = statement.query_map([project_id], map_project_suggestion)?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(ProjectDetail {
            project,
            members,
            suggestions,
        })
    }

    pub fn update_project(
        &self,
        project_id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Project, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let existing: Project = transaction
            .query_row(
                "SELECT id, name, description, status, decision, created_at, updated_at
                 FROM projects WHERE id = ?1",
                [project_id],
                map_project,
            )
            .optional()?
            .ok_or(ChronicleError::ProjectNotFound)?;

        let new_name = name.map(str::trim).filter(|value| !value.is_empty());
        let new_description = description.map(str::trim).map(str::to_owned);
        let final_name = new_name.unwrap_or(&existing.name);
        let final_description = new_description.as_deref().unwrap_or(&existing.description);

        transaction.execute(
            "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![final_name, final_description, now, project_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_project(project_id).map(|detail| detail.project)
    }

    pub fn accept_project(&self, project_id: i64) -> Result<Project, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE projects
             SET status = 'active', decision = 'accepted', updated_at = ?2
             WHERE id = ?1",
            params![project_id, now],
        )?;
        if transaction.changes() == 0 {
            return Err(ChronicleError::ProjectNotFound);
        }
        transaction.execute(
            "UPDATE project_members
             SET membership_type = 'manual'
             WHERE project_id = ?1 AND membership_type = 'suggested'",
            [project_id],
        )?;
        transaction.execute(
            "UPDATE project_suggestions SET handled = 1 WHERE project_id = ?1",
            [project_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_project(project_id).map(|detail| detail.project)
    }

    pub fn reject_project(&self, project_id: i64) -> Result<Project, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE projects
             SET status = 'rejected', decision = 'rejected', updated_at = ?2
             WHERE id = ?1",
            params![project_id, now],
        )?;
        if transaction.changes() == 0 {
            return Err(ChronicleError::ProjectNotFound);
        }
        transaction.execute(
            "UPDATE project_suggestions SET handled = 1 WHERE project_id = ?1",
            [project_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_project(project_id).map(|detail| detail.project)
    }

    pub fn add_project_member(
        &self,
        project_id: i64,
        file_id: i64,
    ) -> Result<ProjectDetail, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let project_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            [project_id],
            |row| row.get(0),
        )?;
        if !project_exists {
            return Err(ChronicleError::ProjectNotFound);
        }
        let result = transaction.execute(
            "INSERT INTO project_members (project_id, file_id, membership_type, added_at)
             VALUES (?1, ?2, 'manual', ?3)",
            params![project_id, file_id, now],
        );
        match result {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(ref code, _)) if code.extended_code == 2067 => {
                return Err(ChronicleError::DuplicateProjectMember);
            }
            Err(error) => return Err(error.into()),
        }
        transaction.execute(
            "UPDATE project_suggestions SET handled = 1 WHERE project_id = ?1 AND file_id = ?2",
            params![project_id, file_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_project(project_id)
    }

    pub fn remove_project_member(
        &self,
        project_id: i64,
        file_id: i64,
    ) -> Result<ProjectDetail, ChronicleError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let deleted = transaction.execute(
            "DELETE FROM project_members WHERE project_id = ?1 AND file_id = ?2",
            params![project_id, file_id],
        )?;
        if deleted == 0 {
            return Err(ChronicleError::ProjectMemberNotFound);
        }
        transaction.execute(
            "UPDATE project_suggestions SET handled = 1 WHERE project_id = ?1 AND file_id = ?2",
            params![project_id, file_id],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_project(project_id)
    }

    pub fn replace_suggested_projects(
        &self,
        groups: Vec<SuggestedProjectGroup>,
    ) -> Result<usize, ChronicleError> {
        if groups.is_empty() {
            return Ok(0);
        }
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;

        transaction.execute(
            "DELETE FROM project_suggestions WHERE project_id IN (
                SELECT id FROM projects WHERE status = 'suggested'
             )",
            [],
        )?;
        transaction.execute(
            "DELETE FROM project_members WHERE project_id IN (
                SELECT id FROM projects WHERE status = 'suggested'
             )",
            [],
        )?;
        transaction.execute("DELETE FROM projects WHERE status = 'suggested'", [])?;

        let mut created = 0_usize;
        for (name, members) in groups {
            if members.len() < 2 {
                continue;
            }
            transaction.execute(
                "INSERT INTO projects (name, description, status, decision, created_at, updated_at)
                 VALUES (?1, '', 'suggested', NULL, ?2, ?2)",
                params![name, now],
            )?;
            let project_id = transaction.last_insert_rowid();
            created += 1;

            let file_ids: HashSet<i64> =
                members.iter().map(|(file_id, _, _, _)| *file_id).collect();
            for file_id in file_ids {
                transaction.execute(
                    "INSERT INTO project_members (project_id, file_id, membership_type, added_at)
                     VALUES (?1, ?2, 'suggested', ?3)",
                    params![project_id, file_id, now],
                )?;
            }
            for (file_id, confidence, evidence, source) in members {
                transaction.execute(
                    "INSERT INTO project_suggestions
                     (project_id, file_id, confidence, evidence, source, handled, suggested_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
                    params![project_id, file_id, confidence, evidence, source, now],
                )?;
            }
        }

        transaction.commit()?;
        Ok(created)
    }

    pub fn list_project_timeline_events(
        &self,
        project_id: i64,
        limit: u32,
    ) -> Result<Vec<TimelineItem>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT e.id, e.file_id, e.indexed_folder_id, e.event_type, e.detected_at,
                    e.filesystem_time, e.old_path, e.new_path, e.confidence, e.event_source,
                    e.user_confirmation, e.user_confirmed_at,
                    f.id, f.indexed_folder_id, f.normalized_path, f.name, f.parent_path,
                    f.extension, f.size_bytes, f.filesystem_created_at, f.filesystem_modified_at,
                    f.first_indexed_at, f.last_seen_at, f.is_present,
                    i.display_name, i.normalized_path
             FROM file_events e
             JOIN files f ON f.id = e.file_id
             JOIN indexed_folders i ON i.id = e.indexed_folder_id
             WHERE e.file_id IN (SELECT file_id FROM project_members WHERE project_id = ?1)
             ORDER BY e.id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![project_id, limit], |row| {
            Ok(TimelineItem {
                event: FileEvent {
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
                    user_confirmation: row.get(10)?,
                    user_confirmed_at: row.get(11)?,
                },
                file: FileRecord {
                    id: row.get(12)?,
                    indexed_folder_id: row.get(13)?,
                    normalized_path: row.get(14)?,
                    name: row.get(15)?,
                    parent_path: row.get(16)?,
                    extension: row.get(17)?,
                    size_bytes: row.get(18)?,
                    filesystem_created_at: row.get(19)?,
                    filesystem_modified_at: row.get(20)?,
                    first_indexed_at: row.get(21)?,
                    last_seen_at: row.get(22)?,
                    is_present: row.get::<_, i64>(23)? != 0,
                },
                folder_name: row.get(24)?,
                folder_path: row.get(25)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn replace_activity_sessions(
        &self,
        sessions: Vec<SessionTuple>,
    ) -> Result<usize, ChronicleError> {
        if sessions.is_empty() {
            return Ok(0);
        }
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;

        transaction.execute(
            "DELETE FROM activity_session_events WHERE session_id IN (
                SELECT id FROM activity_sessions WHERE status = 'auto'
             )",
            [],
        )?;
        transaction.execute(
            "DELETE FROM activity_session_files WHERE session_id IN (
                SELECT id FROM activity_sessions WHERE status = 'auto'
             )",
            [],
        )?;
        transaction.execute("DELETE FROM activity_sessions WHERE status = 'auto'", [])?;

        let mut created = 0_usize;
        for (title, summary, project_id, event_ids, file_ids) in sessions {
            if event_ids.is_empty() {
                continue;
            }
            let started_at = transaction.query_row(
                "SELECT detected_at FROM file_events WHERE id = ?1",
                [event_ids[0]],
                |row| row.get::<_, String>(0),
            )?;
            let ended_at = transaction.query_row(
                "SELECT detected_at FROM file_events WHERE id = ?1",
                [event_ids[event_ids.len() - 1]],
                |row| row.get::<_, String>(0),
            )?;
            transaction.execute(
                "INSERT INTO activity_sessions
                 (project_id, title, started_at, ended_at, event_summary, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'auto', ?6, ?6)",
                params![project_id, title, started_at, ended_at, summary, now],
            )?;
            let session_id = transaction.last_insert_rowid();
            created += 1;

            for event_id in event_ids {
                transaction.execute(
                    "INSERT INTO activity_session_events (session_id, file_event_id)
                     VALUES (?1, ?2)",
                    params![session_id, event_id],
                )?;
            }
            for file_id in file_ids {
                transaction.execute(
                    "INSERT INTO activity_session_files (session_id, file_id)
                     VALUES (?1, ?2)",
                    params![session_id, file_id],
                )?;
            }
        }

        transaction.commit()?;
        Ok(created)
    }

    pub fn list_sessions(
        &self,
        request: &ListSessionsRequest,
    ) -> Result<Vec<ActivitySessionSummary>, ChronicleError> {
        let sessions: Vec<ActivitySession> = {
            let connection = self.connection()?;
            let sql = if request.project_id.is_some() {
                "SELECT s.id, s.project_id, s.title, s.started_at, s.ended_at, s.event_summary,
                        s.status, s.created_at, s.updated_at
                 FROM activity_sessions s
                 WHERE s.project_id = ?1
                 ORDER BY s.started_at DESC, s.id DESC"
            } else {
                "SELECT s.id, s.project_id, s.title, s.started_at, s.ended_at, s.event_summary,
                        s.status, s.created_at, s.updated_at
                 FROM activity_sessions s
                 ORDER BY s.started_at DESC, s.id DESC"
            };
            let mut statement = connection.prepare(sql)?;
            let rows = if let Some(project_id) = request.project_id {
                statement.query_map([project_id], map_activity_session)?
            } else {
                statement.query_map([], map_activity_session)?
            };
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let mut summaries = Vec::with_capacity(sessions.len());
        for session in sessions {
            let event_count = self.session_event_count(session.id)?;
            let file_count = self.session_file_count(session.id)?;
            let project_name = session
                .project_id
                .and_then(|project_id| self.project_name(project_id).ok());
            summaries.push(ActivitySessionSummary {
                session,
                event_count,
                file_count,
                project_name,
            });
        }
        Ok(summaries)
    }

    fn session_event_count(&self, session_id: i64) -> Result<usize, ChronicleError> {
        let connection = self.connection()?;
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM activity_session_events WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    fn session_file_count(&self, session_id: i64) -> Result<usize, ChronicleError> {
        let connection = self.connection()?;
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM activity_session_files WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    pub(crate) fn project_name(&self, project_id: i64) -> Result<String, ChronicleError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT name FROM projects WHERE id = ?1",
                [project_id],
                |row| row.get::<_, String>(0),
            )
            .map_err(Into::into)
    }

    pub fn get_session(&self, session_id: i64) -> Result<ActivitySessionDetail, ChronicleError> {
        let connection = self.connection()?;
        let session: ActivitySession = connection
            .query_row(
                "SELECT id, project_id, title, started_at, ended_at, event_summary,
                        status, created_at, updated_at
                 FROM activity_sessions WHERE id = ?1",
                [session_id],
                map_activity_session,
            )
            .optional()?
            .ok_or(ChronicleError::SessionNotFound)?;

        let project = if let Some(project_id) = session.project_id {
            connection
                .query_row(
                    "SELECT id, name, description, status, decision, created_at, updated_at
                     FROM projects WHERE id = ?1",
                    [project_id],
                    map_project,
                )
                .optional()?
        } else {
            None
        };

        let events = {
            let mut statement = connection.prepare(
                "SELECT e.id, e.file_id, e.indexed_folder_id, e.event_type, e.detected_at,
                        e.filesystem_time, e.old_path, e.new_path, e.confidence, e.event_source,
                        e.user_confirmation, e.user_confirmed_at
                 FROM activity_session_events se
                 JOIN file_events e ON e.id = se.file_event_id
                 WHERE se.session_id = ?1
                 ORDER BY e.detected_at ASC, e.id ASC",
            )?;
            let rows = statement.query_map([session_id], |row| {
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
                    user_confirmation: row.get(10)?,
                    user_confirmed_at: row.get(11)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let files = {
            let mut statement = connection.prepare(
                "SELECT f.id, f.indexed_folder_id, f.normalized_path, f.name, f.parent_path,
                        f.extension, f.size_bytes, f.filesystem_created_at,
                        f.filesystem_modified_at, f.first_indexed_at, f.last_seen_at, f.is_present
                 FROM activity_session_files sf
                 JOIN files f ON f.id = sf.file_id
                 WHERE sf.session_id = ?1
                 ORDER BY f.name ASC",
            )?;
            let rows = statement.query_map([session_id], |row| {
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
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(ActivitySessionDetail {
            session,
            project,
            files,
            events,
        })
    }

    pub fn update_session(
        &self,
        session_id: i64,
        title: Option<&str>,
        project_id: Option<i64>,
    ) -> Result<ActivitySession, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let existing: ActivitySession = transaction
            .query_row(
                "SELECT id, project_id, title, started_at, ended_at, event_summary,
                        status, created_at, updated_at
                 FROM activity_sessions WHERE id = ?1",
                [session_id],
                map_activity_session,
            )
            .optional()?
            .ok_or(ChronicleError::SessionNotFound)?;

        let final_title = title.map(str::trim).filter(|value| !value.is_empty());
        let final_project_id = if title.is_some() && project_id.is_none() {
            existing.project_id
        } else {
            project_id
        };
        let new_status = if title.is_some() || project_id.is_some() {
            ActivitySessionStatus::Edited
        } else {
            existing.status
        };

        transaction.execute(
            "UPDATE activity_sessions
             SET title = ?1, project_id = ?2, status = ?3, updated_at = ?4
             WHERE id = ?5",
            params![
                final_title.unwrap_or(&existing.title),
                final_project_id,
                new_status.as_str(),
                now,
                session_id
            ],
        )?;
        if transaction.changes() == 0 {
            return Err(ChronicleError::SessionNotFound);
        }
        transaction.commit()?;
        drop(connection);
        self.get_session(session_id).map(|detail| detail.session)
    }

    pub fn accept_session(&self, session_id: i64) -> Result<ActivitySession, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE activity_sessions
             SET status = 'accepted', updated_at = ?2
             WHERE id = ?1",
            params![session_id, now],
        )?;
        if transaction.changes() == 0 {
            return Err(ChronicleError::SessionNotFound);
        }
        transaction.commit()?;
        drop(connection);
        self.get_session(session_id).map(|detail| detail.session)
    }

    pub fn reject_session(&self, session_id: i64) -> Result<ActivitySession, ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE activity_sessions
             SET status = 'rejected', updated_at = ?2
             WHERE id = ?1",
            params![session_id, now],
        )?;
        if transaction.changes() == 0 {
            return Err(ChronicleError::SessionNotFound);
        }
        transaction.commit()?;
        drop(connection);
        self.get_session(session_id).map(|detail| detail.session)
    }

    pub fn set_folder_content_indexing(
        &self,
        folder_id: i64,
        enabled: bool,
        extensions: Option<&str>,
        max_bytes: Option<i64>,
        exclusion_patterns: Option<&str>,
    ) -> Result<IndexedFolder, ChronicleError> {
        let connection = self.connection()?;
        if let Some(value) = extensions {
            connection.execute(
                "UPDATE indexed_folders SET content_indexing_extensions = ?2 WHERE id = ?1",
                params![folder_id, value],
            )?;
        }
        if let Some(value) = max_bytes {
            connection.execute(
                "UPDATE indexed_folders SET content_indexing_max_bytes = ?2 WHERE id = ?1",
                params![folder_id, value],
            )?;
        }
        if let Some(value) = exclusion_patterns {
            connection.execute(
                "UPDATE indexed_folders SET content_indexing_exclusion_patterns = ?2 WHERE id = ?1",
                params![folder_id, value],
            )?;
        }
        connection.execute(
            "UPDATE indexed_folders SET content_indexing_enabled = ?2 WHERE id = ?1",
            params![folder_id, if enabled { 1 } else { 0 }],
        )?;
        drop(connection);
        self.get_indexed_folder(folder_id)
    }

    pub fn list_present_files_for_content_indexing(
        &self,
        folder_id: i64,
    ) -> Result<Vec<FileRecord>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    size_bytes, filesystem_created_at, filesystem_modified_at,
                    first_indexed_at, last_seen_at, is_present
             FROM files
             WHERE indexed_folder_id = ?1 AND is_present = 1",
        )?;
        let rows = statement.query_map([folder_id], map_content_file_record)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn content_indexed_documents_for_folder(
        &self,
        folder_id: i64,
    ) -> Result<Vec<(i64, i64, String, String)>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT d.file_id, d.size_bytes, d.indexed_at, f.filesystem_modified_at
             FROM content_index_documents d
             JOIN files f ON f.id = d.file_id
             WHERE f.indexed_folder_id = ?1 AND f.is_present = 1",
        )?;
        let rows = statement.query_map([folder_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn content_index_document(
        &self,
        file_id: i64,
        content: &str,
        size_bytes: i64,
        word_count: usize,
    ) -> Result<(), ChronicleError> {
        let now = now_rfc3339();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM content_index_fts WHERE doc_id = ?1", [file_id])?;
        transaction.execute(
            "DELETE FROM content_index_documents WHERE file_id = ?1",
            [file_id],
        )?;
        transaction.execute(
            "INSERT INTO content_index_fts (doc_id, content) VALUES (?1, ?2)",
            params![file_id, content],
        )?;
        transaction.execute(
            "INSERT INTO content_index_documents
             (file_id, indexed_at, size_bytes, word_count)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                file_id,
                now,
                size_bytes,
                i64::try_from(word_count).unwrap_or(0)
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn delete_content_index_for_file(&self, file_id: i64) -> Result<(), ChronicleError> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM content_index_fts WHERE doc_id = ?1", [file_id])?;
        connection.execute(
            "DELETE FROM content_index_documents WHERE file_id = ?1",
            [file_id],
        )?;
        Ok(())
    }

    pub fn delete_content_index_for_folder(&self, folder_id: i64) -> Result<(), ChronicleError> {
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM content_index_fts WHERE doc_id IN (SELECT id FROM files WHERE indexed_folder_id = ?1)",
            [folder_id],
        )?;
        connection.execute(
            "DELETE FROM content_index_documents WHERE file_id IN (SELECT id FROM files WHERE indexed_folder_id = ?1)",
            [folder_id],
        )?;
        Ok(())
    }

    pub fn clear_content_index(&self) -> Result<(), ChronicleError> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM content_index_fts", [])?;
        connection.execute("DELETE FROM content_index_documents", [])?;
        Ok(())
    }

    pub fn search_content(
        &self,
        query: &str,
        folder_id: Option<i64>,
        limit: u32,
    ) -> Result<Vec<ContentSearchResult>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT f.id, f.indexed_folder_id, f.normalized_path, f.name, f.parent_path,
                    f.extension, f.size_bytes, f.filesystem_created_at, f.filesystem_modified_at,
                    f.first_indexed_at, f.last_seen_at, f.is_present,
                    snippet(content_index_fts, 0, '<<<', '>>>', '...', 30) AS snippet,
                    rank
             FROM content_index_fts
             JOIN files f ON f.id = content_index_fts.doc_id
             WHERE content_index_fts MATCH ?1 AND f.is_present = 1
               AND (?2 IS NULL OR f.indexed_folder_id = ?2)
             ORDER BY rank
             LIMIT ?3",
        )?;
        let rows = statement.query_map(params![query, folder_id, i64::from(limit)], |row| {
            let file = FileRecord {
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
            };
            let snippet: String = row.get(12)?;
            let rank: f64 = row.get(13)?;
            Ok(ContentSearchResult {
                file,
                snippet,
                rank,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn search_filename(
        &self,
        query: &str,
        folder_id: Option<i64>,
        limit: u32,
    ) -> Result<Vec<FileRecord>, ChronicleError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, indexed_folder_id, normalized_path, name, parent_path, extension,
                    size_bytes, filesystem_created_at, filesystem_modified_at,
                    first_indexed_at, last_seen_at, is_present
             FROM files
             WHERE is_present = 1 AND name LIKE ?1
               AND (?2 IS NULL OR indexed_folder_id = ?2)
             ORDER BY id
             LIMIT ?3",
        )?;
        let pattern = format!("%{query}%");
        let rows = statement.query_map(
            params![pattern, folder_id, i64::from(limit)],
            map_content_file_record,
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

fn map_content_file_record(row: &rusqlite::Row<'_>) -> Result<FileRecord, rusqlite::Error> {
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
}
