use tauri::State;

use crate::{
    database::Database,
    models::{ApplicationError, TimelinePage, TimelineRequest},
};

pub fn query_timeline_page_impl(
    database: &Database,
    request: TimelineRequest,
) -> Result<TimelinePage, ApplicationError> {
    database
        .query_timeline_page(request.cursor, request.page_size)
        .map_err(ApplicationError::from)
}

#[tauri::command]
pub fn query_timeline_page(
    database: State<'_, Database>,
    request: TimelineRequest,
) -> Result<TimelinePage, ApplicationError> {
    query_timeline_page_impl(database.inner(), request)
}
