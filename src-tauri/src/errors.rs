use std::io;

use thiserror::Error;

use crate::models::ApplicationError;

#[derive(Debug, Error)]
pub enum ChronicleError {
    #[error("database operation failed")]
    Database(#[from] rusqlite::Error),
    #[error("database migration failed")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("local storage operation failed")]
    Io(#[from] io::Error),
    #[error("database state is unavailable")]
    DatabaseState,
    #[error("the selected path is not a directory")]
    NotDirectory,
    #[error("the selected path is a symbolic link or reparse point")]
    SymbolicLinkRoot,
    #[error("the path cannot be represented safely")]
    PathEncoding,
    #[error("the folder is already indexed")]
    DuplicateFolder,
    #[error("the folder is not registered")]
    FolderNotFound,
    #[error("the folder is missing or moved")]
    MissingOrMoved,
    #[error("permission was denied")]
    PermissionDenied,
    #[error("the folder is inaccessible")]
    Inaccessible,
    #[error("explicit removal confirmation is required")]
    ConfirmationRequired,
    #[error("the scan run is not available")]
    ScanNotFound,
    #[error("a scan is already running for this folder")]
    ScanAlreadyRunning,
    #[error("monitoring is already running for this folder")]
    WatcherAlreadyRunning,
    #[error("monitoring is not running for this folder")]
    WatcherNotRunning,
    #[error("filesystem watcher failed")]
    WatcherFailed,
    #[error("a filesystem event escaped the authorized root")]
    UnauthorizedEventPath,
    #[error("too many filesystem events arrived at once")]
    EventStorm,
    #[error("the scan was cancelled")]
    Cancelled,
    #[error("the file is deleted or no longer available")]
    FileUnavailable,
    #[error("a numeric value exceeded the supported range")]
    NumericOverflow,
}

impl From<ChronicleError> for ApplicationError {
    fn from(error: ChronicleError) -> Self {
        match error {
            ChronicleError::Database(_)
            | ChronicleError::Migration(_)
            | ChronicleError::DatabaseState => ApplicationError::database_unavailable(),
            ChronicleError::DuplicateFolder => {
                ApplicationError::new("duplicate_folder", "errors.duplicateFolder", false)
            }
            ChronicleError::NotDirectory => {
                ApplicationError::new("not_directory", "errors.notDirectory", false)
            }
            ChronicleError::SymbolicLinkRoot => {
                ApplicationError::new("symbolic_link_root", "errors.symbolicLinkRoot", false)
            }
            ChronicleError::FolderNotFound => {
                ApplicationError::new("folder_not_found", "errors.folderNotFound", false)
            }
            ChronicleError::MissingOrMoved => {
                ApplicationError::new("missing_or_moved", "errors.missingOrMoved", true)
            }
            ChronicleError::PermissionDenied => {
                ApplicationError::new("permission_denied", "errors.permissionDenied", true)
            }
            ChronicleError::Inaccessible | ChronicleError::Io(_) => {
                ApplicationError::new("folder_inaccessible", "errors.folderInaccessible", true)
            }
            ChronicleError::ConfirmationRequired => ApplicationError::new(
                "confirmation_required",
                "errors.confirmationRequired",
                false,
            ),
            ChronicleError::ScanNotFound => {
                ApplicationError::new("scan_not_found", "errors.scanNotFound", false)
            }
            ChronicleError::ScanAlreadyRunning => {
                ApplicationError::new("scan_already_running", "errors.scanAlreadyRunning", false)
            }
            ChronicleError::WatcherAlreadyRunning => ApplicationError::new(
                "watcher_already_running",
                "errors.watcherAlreadyRunning",
                false,
            ),
            ChronicleError::WatcherNotRunning => {
                ApplicationError::new("watcher_not_running", "errors.watcherNotRunning", false)
            }
            ChronicleError::WatcherFailed => {
                ApplicationError::new("watcher_failed", "errors.watcherFailed", true)
            }
            ChronicleError::UnauthorizedEventPath => ApplicationError::new(
                "unauthorized_event_path",
                "errors.unauthorizedEventPath",
                false,
            ),
            ChronicleError::EventStorm => {
                ApplicationError::new("event_storm", "errors.eventStorm", true)
            }
            ChronicleError::Cancelled => {
                ApplicationError::new("scan_cancelled", "errors.scanCancelled", false)
            }
            ChronicleError::FileUnavailable => {
                ApplicationError::new("file_unavailable", "errors.fileUnavailable", false)
            }
            ChronicleError::PathEncoding | ChronicleError::NumericOverflow => {
                ApplicationError::unexpected()
            }
        }
    }
}
