use tauri::State;

use crate::{
    database::Database,
    models::{
        ApplicationError, ConfirmEventRequest, FileActionRequest, FileEvent, FileHistoryRequest,
        PathHistoryItem, PathHistoryRequest, ScanHistoryRequest, ScanRun, TimelinePage,
        TimelineRequest,
    },
    platform::{open_path, resolve_authorized_file, reveal_path},
};

pub fn query_timeline_page_impl(
    database: &Database,
    request: TimelineRequest,
) -> Result<TimelinePage, ApplicationError> {
    database
        .query_timeline_page(&request)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn get_file_event_history(
    database: State<'_, Database>,
    request: FileHistoryRequest,
) -> Result<Vec<FileEvent>, ApplicationError> {
    database
        .file_event_history(request.file_id, request.limit)
        .map_err(Into::into)
}

#[tauri::command]
pub fn get_scan_history(
    database: State<'_, Database>,
    request: ScanHistoryRequest,
) -> Result<Vec<ScanRun>, ApplicationError> {
    database
        .scan_history(request.folder_id, request.limit)
        .map_err(Into::into)
}

#[tauri::command]
pub fn get_file_path_history(
    database: State<'_, Database>,
    request: PathHistoryRequest,
) -> Result<Vec<PathHistoryItem>, ApplicationError> {
    database
        .file_path_history(request.file_id, request.limit)
        .map_err(Into::into)
}

#[tauri::command]
pub fn confirm_event(
    database: State<'_, Database>,
    request: ConfirmEventRequest,
) -> Result<(), ApplicationError> {
    database.confirm_event(request.event_id).map_err(Into::into)
}

#[tauri::command]
pub fn reject_event(
    database: State<'_, Database>,
    request: ConfirmEventRequest,
) -> Result<(), ApplicationError> {
    database.reject_event(request.event_id).map_err(Into::into)
}

#[tauri::command]
pub fn open_timeline_file(
    database: State<'_, Database>,
    request: FileActionRequest,
) -> Result<(), ApplicationError> {
    let (path, root) = database
        .present_file_context(request.file_id)
        .map_err(ApplicationError::from)?;
    let resolved =
        resolve_authorized_file(std::path::Path::new(&path), std::path::Path::new(&root))
            .map_err(ApplicationError::from)?;
    open_path(&resolved).map_err(Into::into)
}

#[tauri::command]
pub fn reveal_timeline_file(
    database: State<'_, Database>,
    request: FileActionRequest,
) -> Result<(), ApplicationError> {
    let (path, root) = database
        .present_file_context(request.file_id)
        .map_err(ApplicationError::from)?;
    let resolved =
        resolve_authorized_file(std::path::Path::new(&path), std::path::Path::new(&root))
            .map_err(ApplicationError::from)?;
    reveal_path(&resolved).map_err(Into::into)
}

#[tauri::command]
pub fn query_timeline_page(
    database: State<'_, Database>,
    request: TimelineRequest,
) -> Result<TimelinePage, ApplicationError> {
    query_timeline_page_impl(database.inner(), request)
}

#[tauri::command]
pub fn toggle_event_favorite(
    database: State<'_, Database>,
    event_id: i64,
) -> Result<bool, ApplicationError> {
    database.toggle_event_favorite(event_id).map_err(Into::into)
}

#[tauri::command]
pub fn list_favorite_event_ids(
    database: State<'_, Database>,
) -> Result<Vec<i64>, ApplicationError> {
    database.list_favorite_event_ids().map_err(Into::into)
}
