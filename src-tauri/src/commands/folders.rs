use tauri::State;

use crate::{
    database::Database,
    folders,
    models::{
        ApplicationError, FolderRegistration, IndexedFolder, RemoveIndexedFolderRequest,
        ScanTaskSnapshot,
    },
    scanner,
    tasks::ScanTaskManager,
};

pub fn list_indexed_folders_impl(
    database: &Database,
) -> Result<Vec<IndexedFolder>, ApplicationError> {
    folders::list_folders(database).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_indexed_folders(
    database: State<'_, Database>,
) -> Result<Vec<IndexedFolder>, ApplicationError> {
    list_indexed_folders_impl(database.inner())
}

#[tauri::command]
pub fn register_indexed_folder(
    database: State<'_, Database>,
    path: String,
) -> Result<FolderRegistration, ApplicationError> {
    folders::register_folder(database.inner(), &path).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn remove_indexed_folder(
    database: State<'_, Database>,
    request: RemoveIndexedFolderRequest,
) -> Result<(), ApplicationError> {
    folders::remove_folder(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn start_folder_scan(
    database: State<'_, Database>,
    tasks: State<'_, ScanTaskManager>,
    folder_id: i64,
) -> Result<ScanTaskSnapshot, ApplicationError> {
    scanner::start_scan(database.inner().clone(), tasks.inner().clone(), folder_id)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn get_scan_task(
    database: State<'_, Database>,
    scan_run_id: i64,
) -> Result<ScanTaskSnapshot, ApplicationError> {
    database
        .scan_task_snapshot(scan_run_id)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn cancel_folder_scan(
    tasks: State<'_, ScanTaskManager>,
    scan_run_id: i64,
) -> Result<(), ApplicationError> {
    tasks.cancel(scan_run_id).map_err(ApplicationError::from)
}
