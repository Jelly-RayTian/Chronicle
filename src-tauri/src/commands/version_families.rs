use tauri::State;

use crate::{
    database::Database,
    models::{
        AcceptVersionFamilyRequest, AddVersionFamilyMemberRequest, ApplicationError,
        ListVersionFamiliesRequest, MergeVersionFamiliesRequest, RejectVersionFamilyRequest,
        RemoveVersionFamilyMemberRequest, RenameVersionFamilyRequest, SplitVersionFamilyRequest,
        SuggestVersionFamiliesRequest, SuggestVersionFamiliesResponse, VersionFamily,
        VersionFamilyDetail, VersionFamilyRequest, VersionFamilySummary,
    },
    version_families,
};

pub fn list_version_families_impl(
    database: &Database,
    request: ListVersionFamiliesRequest,
) -> Result<Vec<VersionFamilySummary>, ApplicationError> {
    version_families::list_version_families(database, &request).map_err(ApplicationError::from)
}

#[tauri::command]
pub fn list_version_families(
    database: State<'_, Database>,
    request: ListVersionFamiliesRequest,
) -> Result<Vec<VersionFamilySummary>, ApplicationError> {
    list_version_families_impl(database.inner(), request)
}

#[tauri::command]
pub fn get_version_family(
    database: State<'_, Database>,
    request: VersionFamilyRequest,
) -> Result<VersionFamilyDetail, ApplicationError> {
    version_families::get_version_family(database.inner(), &request).map_err(Into::into)
}

#[tauri::command]
pub fn suggest_version_families(
    database: State<'_, Database>,
    request: SuggestVersionFamiliesRequest,
) -> Result<SuggestVersionFamiliesResponse, ApplicationError> {
    version_families::suggest_version_families(database.inner(), request.folder_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn accept_version_family(
    database: State<'_, Database>,
    request: AcceptVersionFamilyRequest,
) -> Result<VersionFamily, ApplicationError> {
    version_families::accept_version_family(
        database.inner(),
        &crate::models::VersionFamilyRequest {
            family_id: request.family_id,
        },
    )
    .map_err(Into::into)
}

#[tauri::command]
pub fn reject_version_family(
    database: State<'_, Database>,
    request: RejectVersionFamilyRequest,
) -> Result<VersionFamily, ApplicationError> {
    version_families::reject_version_family(
        database.inner(),
        &crate::models::VersionFamilyRequest {
            family_id: request.family_id,
        },
    )
    .map_err(Into::into)
}

#[tauri::command]
pub fn rename_version_family(
    database: State<'_, Database>,
    request: RenameVersionFamilyRequest,
) -> Result<VersionFamily, ApplicationError> {
    version_families::rename_version_family(database.inner(), &request).map_err(Into::into)
}

#[tauri::command]
pub fn split_version_family(
    database: State<'_, Database>,
    request: SplitVersionFamilyRequest,
) -> Result<VersionFamilyDetail, ApplicationError> {
    version_families::split_version_family(database.inner(), &request).map_err(Into::into)
}

#[tauri::command]
pub fn merge_version_families(
    database: State<'_, Database>,
    request: MergeVersionFamiliesRequest,
) -> Result<VersionFamilyDetail, ApplicationError> {
    version_families::merge_version_families(database.inner(), &request).map_err(Into::into)
}

#[tauri::command]
pub fn add_version_family_member(
    database: State<'_, Database>,
    request: AddVersionFamilyMemberRequest,
) -> Result<VersionFamilyDetail, ApplicationError> {
    version_families::add_version_family_member(database.inner(), &request).map_err(Into::into)
}

#[tauri::command]
pub fn remove_version_family_member(
    database: State<'_, Database>,
    request: RemoveVersionFamilyMemberRequest,
) -> Result<VersionFamilyDetail, ApplicationError> {
    version_families::remove_version_family_member(database.inner(), &request).map_err(Into::into)
}
