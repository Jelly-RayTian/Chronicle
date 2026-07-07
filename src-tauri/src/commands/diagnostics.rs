use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::{
    commands::{application::get_application_info_impl, database::get_database_status_impl},
    database::Database,
    models::{ApplicationError, DiagnosticsReport},
};

#[tauri::command]
pub async fn export_diagnostics(
    app: AppHandle,
    database: State<'_, Database>,
) -> Result<Option<String>, ApplicationError> {
    let Some(path) = app
        .dialog()
        .file()
        .set_file_name("chronicle-diagnostics.json")
        .add_filter("JSON", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };

    let path = path
        .into_path()
        .map_err(|error| ApplicationError::new("diagnostics_path", error.to_string(), false))?;
    let report = build_report(database.inner())?;
    let serialized = serde_json::to_string_pretty(&report).map_err(|error| {
        ApplicationError::new("diagnostics_serialization", error.to_string(), false)
    })?;
    std::fs::write(&path, serialized)
        .map_err(|error| ApplicationError::new("diagnostics_write", error.to_string(), false))?;
    path.to_str()
        .map(|value| value.to_owned())
        .ok_or_else(|| ApplicationError::new("diagnostics_path", "invalid path".to_owned(), false))
        .map(Some)
}

fn build_report(database: &Database) -> Result<DiagnosticsReport, ApplicationError> {
    let application = get_application_info_impl();
    let database_status = get_database_status_impl(database).map_err(|error| {
        ApplicationError::new("diagnostics_status", error.message_key, error.retryable)
    })?;
    let counts = database
        .diagnostic_counts()
        .map_err(|error| ApplicationError::new("diagnostics_counts", error.to_string(), false))?;
    Ok(DiagnosticsReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        application,
        database: database_status,
        counts,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn export_diagnostics_writes_sanitized_json() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let report = build_report(&database)
            .unwrap_or_else(|error| panic!("report should build: {error:?}"));
        assert_eq!(report.application.name, "Chronicle");
        assert_eq!(report.counts.indexed_folders, 0);
        assert_eq!(report.counts.content_index_documents, 0);
        let path = directory.path().join("diagnostics.json");
        let json = serde_json::to_string_pretty(&report)
            .unwrap_or_else(|error| panic!("report should serialize: {error}"));
        std::fs::write(&path, json).unwrap_or_else(|error| panic!("file should write: {error}"));
        let mut file =
            std::fs::File::open(&path).unwrap_or_else(|error| panic!("file should open: {error}"));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|error| panic!("file should read: {error}"));
        assert!(contents.contains("Chronicle"));
        assert!(!contents.contains("normalizedPath"));
        assert!(!contents.contains("C:\\"));
    }
}
