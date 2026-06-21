use tauri::State;

use crate::{
    database::Database,
    models::{ApplicationError, IndexedFolder},
};

pub fn list_indexed_folders_impl(
    database: &Database,
) -> Result<Vec<IndexedFolder>, ApplicationError> {
    database
        .list_indexed_folders()
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_indexed_folders(
    database: State<'_, Database>,
) -> Result<Vec<IndexedFolder>, ApplicationError> {
    list_indexed_folders_impl(database.inner())
}
