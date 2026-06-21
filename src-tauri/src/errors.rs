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
}

impl From<ChronicleError> for ApplicationError {
    fn from(_error: ChronicleError) -> Self {
        ApplicationError::database_unavailable()
    }
}
