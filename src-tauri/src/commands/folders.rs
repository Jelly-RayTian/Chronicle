use tauri::State;

use crate::{
    database::Database,
    folders,
    models::{
        ApplicationError, FileRecord, FolderRegistration, IndexedFolder,
        RemoveIndexedFolderRequest, ScanTaskSnapshot,
    },
    scanner,
    tasks::ScanTaskManager,
    watcher::WatcherManager,
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
    watcher: State<'_, WatcherManager>,
    request: RemoveIndexedFolderRequest,
) -> Result<(), ApplicationError> {
    let folder_id = request.folder_id;
    folders::remove_folder(database.inner(), request).map_err(ApplicationError::from)?;
    watcher
        .stop_runtime(folder_id)
        .map_err(ApplicationError::from)
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
    database: State<'_, Database>,
    tasks: State<'_, ScanTaskManager>,
    scan_run_id: i64,
) -> Result<(), ApplicationError> {
    database
        .cancel_scan(scan_run_id)
        .map_err(ApplicationError::from)?;
    tasks.cancel(scan_run_id).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_all_files(database: State<'_, Database>) -> Result<Vec<FileRecord>, ApplicationError> {
    database.list_all_file_records().map_err(Into::into)
}
