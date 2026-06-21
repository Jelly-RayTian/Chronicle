use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AvailabilityStatus {
    Available,
    Missing,
    Unavailable,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub id: i64,
    pub indexed_folder_id: i64,
    pub normalized_path: String,
    pub name: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePage {
    pub items: Vec<FileEvent>,
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
#[serde(rename_all = "camelCase")]
pub struct TimelineRequest {
    pub cursor: Option<i64>,
    pub page_size: u32,
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
