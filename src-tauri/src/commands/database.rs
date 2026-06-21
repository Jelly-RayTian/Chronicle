use tauri::State;

use crate::{
    database::Database,
    models::{ApplicationError, DatabaseState, DatabaseStatus},
};

pub fn get_database_status_impl(database: &Database) -> Result<DatabaseStatus, ApplicationError> {
    let schema_version = database.schema_version().map_err(ApplicationError::from)?;
    Ok(DatabaseStatus {
        state: DatabaseState::Ready,
        schema_version,
    })
}

#[tauri::command]
pub fn get_database_status(
    database: State<'_, Database>,
) -> Result<DatabaseStatus, ApplicationError> {
    get_database_status_impl(database.inner())
}
