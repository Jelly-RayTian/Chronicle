use tauri::State;

use crate::{
    database::Database,
    models::{
        ActivitySession, ActivitySessionDetail, ApplicationError, GenerateSessionsRequest,
        GenerateSessionsResponse, ListSessionsRequest, SessionRequest, UpdateSessionRequest,
    },
    sessions,
};

pub fn list_sessions_impl(
    database: &Database,
    request: ListSessionsRequest,
) -> Result<Vec<crate::models::ActivitySessionSummary>, ApplicationError> {
    sessions::list_sessions(database, request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_sessions(
    database: State<'_, Database>,
    request: ListSessionsRequest,
) -> Result<Vec<crate::models::ActivitySessionSummary>, ApplicationError> {
    list_sessions_impl(database.inner(), request)
}

#[tauri::command]
pub fn get_session(
    database: State<'_, Database>,
    request: SessionRequest,
) -> Result<ActivitySessionDetail, ApplicationError> {
    sessions::get_session(database.inner(), &request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn generate_sessions(
    database: State<'_, Database>,
    request: GenerateSessionsRequest,
) -> Result<GenerateSessionsResponse, ApplicationError> {
    sessions::generate_sessions(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn update_session(
    database: State<'_, Database>,
    request: UpdateSessionRequest,
) -> Result<ActivitySession, ApplicationError> {
    sessions::update_session(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn accept_session(
    database: State<'_, Database>,
    request: SessionRequest,
) -> Result<ActivitySession, ApplicationError> {
    sessions::accept_session(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn reject_session(
    database: State<'_, Database>,
    request: SessionRequest,
) -> Result<ActivitySession, ApplicationError> {
    sessions::reject_session(database.inner(), request).map_err(ApplicationError::from)
}
