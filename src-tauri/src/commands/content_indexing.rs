use tauri::State;

use crate::{
    content_indexing::{self},
    database::Database,
    errors::ChronicleError,
    models::{
        ClearContentIndexRequest, ContentSearchResult, EnableFolderContentIndexingRequest,
        FolderContentIndexingRequest, ReindexFolderContentResponse, SearchFilesRequest,
    },
};

#[tauri::command]
pub fn enable_folder_content_indexing(
    database: State<'_, Database>,
    request: EnableFolderContentIndexingRequest,
) -> Result<crate::models::IndexedFolder, crate::models::ApplicationError> {
    enable_folder_content_indexing_impl(&database, request).map_err(Into::into)
}

fn enable_folder_content_indexing_impl(
    database: &Database,
    request: EnableFolderContentIndexingRequest,
) -> Result<crate::models::IndexedFolder, ChronicleError> {
    let folder = database.get_indexed_folder(request.folder_id)?;
    database.set_folder_content_indexing(
        request.folder_id,
        true,
        request.extensions.as_deref(),
        request.max_bytes,
        request.exclusion_patterns.as_deref(),
    )?;
    let _ = content_indexing::sync_folder(database, request.folder_id);
    database
        .get_indexed_folder(request.folder_id)
        .or(Ok(folder))
}

#[tauri::command]
pub fn disable_folder_content_indexing(
    database: State<'_, Database>,
    request: FolderContentIndexingRequest,
) -> Result<crate::models::IndexedFolder, crate::models::ApplicationError> {
    disable_folder_content_indexing_impl(&database, request).map_err(Into::into)
}

fn disable_folder_content_indexing_impl(
    database: &Database,
    request: FolderContentIndexingRequest,
) -> Result<crate::models::IndexedFolder, ChronicleError> {
    let folder = database.get_indexed_folder(request.folder_id)?;
    content_indexing::remove_folder_content_index(database, request.folder_id)?;
    database
        .set_folder_content_indexing(request.folder_id, false, None, None, None)
        .or(Ok(folder))
}

#[tauri::command]
pub fn reindex_folder_content(
    database: State<'_, Database>,
    request: FolderContentIndexingRequest,
) -> Result<ReindexFolderContentResponse, crate::models::ApplicationError> {
    reindex_folder_content_impl(&database, request).map_err(Into::into)
}

fn reindex_folder_content_impl(
    database: &Database,
    request: FolderContentIndexingRequest,
) -> Result<ReindexFolderContentResponse, ChronicleError> {
    content_indexing::sync_folder(database, request.folder_id)
}

#[tauri::command]
pub fn clear_all_content_index(
    database: State<'_, Database>,
    _request: ClearContentIndexRequest,
) -> Result<(), crate::models::ApplicationError> {
    clear_all_content_index_impl(&database).map_err(Into::into)
}

fn clear_all_content_index_impl(database: &Database) -> Result<(), ChronicleError> {
    content_indexing::clear_all_content_index(database)
}

#[tauri::command]
pub fn search_files(
    database: State<'_, Database>,
    request: SearchFilesRequest,
) -> Result<Vec<ContentSearchResult>, crate::models::ApplicationError> {
    content_indexing::search_files(&database, request).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use crate::{
        commands::content_indexing::{
            clear_all_content_index_impl, disable_folder_content_indexing_impl,
            enable_folder_content_indexing_impl, reindex_folder_content_impl,
        },
        content_indexing::search_files,
        database::Database,
        models::{
            ContentSearchMode, EnableFolderContentIndexingRequest, FolderContentIndexingRequest,
            SearchFilesRequest,
        },
    };

    fn setup() -> (TempDir, Database, std::path::PathBuf, i64) {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let database = Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        (directory, database, root, folder.id)
    }

    #[test]
    fn search_requires_non_empty_query() {
        let (_directory, database, _root, _folder_id) = setup();
        let results = search_files(
            &database,
            SearchFilesRequest {
                query: "   ".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: None,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
                limit: 10,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(results.is_empty());
    }

    #[test]
    fn disabling_content_indexing_clears_folder_rows() {
        let (_directory, database, _root, folder_id) = setup();
        let _ = enable_folder_content_indexing_impl(
            &database,
            EnableFolderContentIndexingRequest {
                folder_id,
                extensions: None,
                max_bytes: None,
                exclusion_patterns: None,
            },
        )
        .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        let disabled = disable_folder_content_indexing_impl(
            &database,
            FolderContentIndexingRequest { folder_id },
        )
        .unwrap_or_else(|error| panic!("disable should succeed: {error}"));
        assert!(!disabled.content_indexing_enabled);
    }

    #[test]
    fn clear_all_content_index_succeeds_on_empty_database() {
        let (_directory, database, _root, _folder_id) = setup();
        clear_all_content_index_impl(&database)
            .unwrap_or_else(|error| panic!("clear should succeed: {error}"));
    }

    #[test]
    fn reindex_on_disabled_folder_returns_zero_counts() {
        let (_directory, database, _root, folder_id) = setup();
        let result =
            reindex_folder_content_impl(&database, FolderContentIndexingRequest { folder_id })
                .unwrap_or_else(|error| panic!("reindex should succeed: {error}"));
        assert_eq!(result.files_indexed, 0);
        assert_eq!(result.files_removed, 0);
    }
}
