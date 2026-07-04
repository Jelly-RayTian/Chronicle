use tauri::State;

use crate::{
    database::Database,
    models::{
        AcceptProjectRequest, AddProjectMemberRequest, ApplicationError, CreateProjectRequest,
        ListProjectsRequest, Project, ProjectDetail, ProjectRequest, ProjectSummary,
        RejectProjectRequest, RemoveProjectMemberRequest, SuggestProjectsRequest,
        SuggestProjectsResponse, UpdateProjectRequest,
    },
    projects,
};

pub fn list_projects_impl(
    database: &Database,
    request: ListProjectsRequest,
) -> Result<Vec<ProjectSummary>, ApplicationError> {
    projects::list_projects(database, request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_projects(
    database: State<'_, Database>,
    request: ListProjectsRequest,
) -> Result<Vec<ProjectSummary>, ApplicationError> {
    list_projects_impl(database.inner(), request)
}

#[tauri::command]
pub fn get_project(
    database: State<'_, Database>,
    request: ProjectRequest,
) -> Result<ProjectDetail, ApplicationError> {
    projects::get_project(database.inner(), &request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn create_project(
    database: State<'_, Database>,
    request: CreateProjectRequest,
) -> Result<ProjectDetail, ApplicationError> {
    projects::create_project(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn update_project(
    database: State<'_, Database>,
    request: UpdateProjectRequest,
) -> Result<Project, ApplicationError> {
    projects::update_project(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn accept_project(
    database: State<'_, Database>,
    request: AcceptProjectRequest,
) -> Result<Project, ApplicationError> {
    projects::accept_project(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn reject_project(
    database: State<'_, Database>,
    request: RejectProjectRequest,
) -> Result<Project, ApplicationError> {
    projects::reject_project(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn add_project_member(
    database: State<'_, Database>,
    request: AddProjectMemberRequest,
) -> Result<ProjectDetail, ApplicationError> {
    projects::add_project_member(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn remove_project_member(
    database: State<'_, Database>,
    request: RemoveProjectMemberRequest,
) -> Result<ProjectDetail, ApplicationError> {
    projects::remove_project_member(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn suggest_projects(
    database: State<'_, Database>,
    request: SuggestProjectsRequest,
) -> Result<SuggestProjectsResponse, ApplicationError> {
    projects::suggest_projects(database.inner(), request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn get_project_timeline(
    database: State<'_, Database>,
    request: ProjectRequest,
) -> Result<Vec<crate::models::TimelineItem>, ApplicationError> {
    database
        .list_project_timeline_events(request.project_id, 250)
        .map_err(ApplicationError::from)
}
