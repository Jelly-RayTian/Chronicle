use tauri::State;

use crate::{
    database::Database,
    models::{ApplicationError, WatcherControlRequest, WatcherStatus},
    watcher::WatcherManager,
};

pub fn list_monitoring_statuses_impl(
    database: &Database,
) -> Result<Vec<WatcherStatus>, ApplicationError> {
    database
        .list_watcher_statuses()
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_monitoring_statuses(
    database: State<'_, Database>,
) -> Result<Vec<WatcherStatus>, ApplicationError> {
    list_monitoring_statuses_impl(database.inner())
}

#[tauri::command]
pub fn get_monitoring_status(
    database: State<'_, Database>,
    watcher: State<'_, WatcherManager>,
    folder_id: i64,
) -> Result<WatcherStatus, ApplicationError> {
    watcher
        .status(database.inner(), folder_id)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn enable_folder_monitoring(
    database: State<'_, Database>,
    watcher: State<'_, WatcherManager>,
    request: WatcherControlRequest,
) -> Result<WatcherStatus, ApplicationError> {
    watcher
        .enable_folder(
            database.inner().clone(),
            request.folder_id,
            request.coalescing_window_ms,
        )
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn disable_folder_monitoring(
    database: State<'_, Database>,
    watcher: State<'_, WatcherManager>,
    request: WatcherControlRequest,
) -> Result<WatcherStatus, ApplicationError> {
    watcher
        .disable_folder(database.inner(), request.folder_id)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn pause_folder_monitoring(
    database: State<'_, Database>,
    watcher: State<'_, WatcherManager>,
    request: WatcherControlRequest,
) -> Result<WatcherStatus, ApplicationError> {
    watcher
        .pause_folder(database.inner(), request.folder_id)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn resume_folder_monitoring(
    database: State<'_, Database>,
    watcher: State<'_, WatcherManager>,
    request: WatcherControlRequest,
) -> Result<WatcherStatus, ApplicationError> {
    watcher
        .resume_folder(database.inner().clone(), request.folder_id)
        .map_err(ApplicationError::from)
}
