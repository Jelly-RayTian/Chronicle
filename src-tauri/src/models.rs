use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    Available,
    MissingOrMoved,
    PermissionDenied,
    Inaccessible,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Idle,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseState {
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFolder {
    pub id: i64,
    pub normalized_path: String,
    pub display_name: String,
    pub added_at: String,
    pub last_successful_scan_at: Option<String>,
    pub monitoring_enabled: bool,
    pub availability_status: AvailabilityStatus,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WatcherDesiredState {
    Disabled,
    Enabled,
    Paused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WatcherRuntimeState {
    Stopped,
    Running,
    Paused,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub folder_id: i64,
    pub desired_state: WatcherDesiredState,
    pub runtime_state: WatcherRuntimeState,
    pub coalescing_window_ms: u64,
    pub last_started_at: Option<String>,
    pub last_stopped_at: Option<String>,
    pub last_event_at: Option<String>,
    pub last_error_at: Option<String>,
    pub last_error_kind: Option<String>,
    pub last_error_message: Option<String>,
    pub events_recorded: u64,
    pub events_dropped: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatcherControlRequest {
    pub folder_id: i64,
    pub coalescing_window_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NestingRelationship {
    InsideExisting,
    ContainsExisting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NestedFolderWarning {
    pub existing_folder_id: i64,
    pub existing_path: String,
    pub relationship: NestingRelationship,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderRegistration {
    pub folder: IndexedFolder,
    pub nested_warnings: Vec<NestedFolderWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub id: i64,
    pub indexed_folder_id: i64,
    pub normalized_path: String,
    pub name: String,
    pub parent_path: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub filesystem_created_at: Option<String>,
    pub filesystem_modified_at: String,
    pub first_indexed_at: String,
    pub last_seen_at: String,
    pub is_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileEvent {
    pub id: i64,
    pub file_id: Option<i64>,
    pub indexed_folder_id: i64,
    pub event_type: String,
    pub detected_at: String,
    pub filesystem_time: Option<String>,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub confidence: Option<f64>,
    pub event_source: String,
    pub user_confirmation: Option<String>,
    pub user_confirmed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RenameConfidence {
    Confirmed,
    StrongInference,
    WeakInference,
    Unknown,
}

impl RenameConfidence {
    #[must_use]
    pub fn confidence(&self) -> f64 {
        match self {
            Self::Confirmed => 1.0,
            Self::StrongInference => 0.8,
            Self::WeakInference => 0.4,
            Self::Unknown => 0.0,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::StrongInference => "strong_inference",
            Self::WeakInference => "weak_inference",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PathHistoryItem {
    pub id: i64,
    pub file_id: i64,
    pub old_path: String,
    pub new_path: String,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub confidence: f64,
    pub evidence: String,
    pub event_source: String,
    pub detected_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmEventRequest {
    pub event_id: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PathHistoryRequest {
    pub file_id: i64,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub event: FileEvent,
    pub file: FileRecord,
    pub folder_name: String,
    pub folder_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanRun {
    pub id: i64,
    pub indexed_folder_id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: TaskStatus,
    pub files_seen: i64,
    pub warning_count: i64,
    pub error_count: i64,
    pub failure_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoveIndexedFolderRequest {
    pub folder_id: i64,
    pub confirm_original_files_untouched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanTaskSnapshot {
    pub scan_run_id: i64,
    pub indexed_folder_id: i64,
    pub status: TaskStatus,
    pub files_seen: u64,
    pub warning_count: u64,
    pub error_count: u64,
    pub cancellable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePage {
    pub items: Vec<TimelineItem>,
    pub next_cursor: Option<i64>,
    pub has_more: bool,
}

impl TimelinePage {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PresenceFilter {
    Present,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRequest {
    pub cursor: Option<i64>,
    pub page_size: u32,
    pub filename: Option<String>,
    pub extension: Option<String>,
    pub event_type: Option<String>,
    pub folder_id: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub presence: Option<PresenceFilter>,
}

impl TimelineRequest {
    #[must_use]
    pub fn first_page(page_size: u32) -> Self {
        Self {
            cursor: None,
            page_size,
            filename: None,
            extension: None,
            event_type: None,
            folder_id: None,
            date_from: None,
            date_to: None,
            presence: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryRequest {
    pub file_id: i64,
    pub limit: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanHistoryRequest {
    pub folder_id: i64,
    pub limit: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileActionRequest {
    pub file_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub state: DatabaseState,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationError {
    pub code: String,
    pub message_key: String,
    pub retryable: bool,
}

impl ApplicationError {
    #[must_use]
    pub fn database_unavailable() -> Self {
        Self {
            code: "database_unavailable".to_owned(),
            message_key: "errors.databaseUnavailable".to_owned(),
            retryable: true,
        }
    }

    #[must_use]
    pub fn unexpected() -> Self {
        Self {
            code: "unexpected_error".to_owned(),
            message_key: "errors.unexpected".to_owned(),
            retryable: false,
        }
    }

    #[must_use]
    pub fn new(code: impl Into<String>, message_key: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message_key: message_key.into(),
            retryable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApplicationError, DatabaseState, DatabaseStatus, TaskStatus, TimelinePage};

    #[test]
    fn empty_timeline_page_serializes_for_the_typescript_boundary() {
        let value = serde_json::to_value(TimelinePage::empty())
            .unwrap_or_else(|error| panic!("timeline page should serialize: {error}"));

        assert_eq!(
            value,
            serde_json::json!({
                "items": [],
                "nextCursor": null,
                "hasMore": false
            })
        );
    }

    #[test]
    fn task_status_serializes_in_lowercase() {
        let value = serde_json::to_value(TaskStatus::Idle)
            .unwrap_or_else(|error| panic!("task status should serialize: {error}"));
        assert_eq!(value, serde_json::json!("idle"));
    }

    #[test]
    fn status_and_errors_use_the_camel_case_command_contract() {
        let status = serde_json::to_value(DatabaseStatus {
            state: DatabaseState::Ready,
            schema_version: 1,
        })
        .unwrap_or_else(|error| panic!("database status should serialize: {error}"));
        let error = serde_json::to_value(ApplicationError::database_unavailable())
            .unwrap_or_else(|source| panic!("application error should serialize: {source}"));

        assert_eq!(
            status,
            serde_json::json!({ "state": "ready", "schemaVersion": 1 })
        );
        assert_eq!(
            error,
            serde_json::json!({
                "code": "database_unavailable",
                "messageKey": "errors.databaseUnavailable",
                "retryable": true
            })
        );
    }
}
