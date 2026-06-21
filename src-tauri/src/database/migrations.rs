use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

use crate::errors::ChronicleError;

const MIGRATION_ARRAY: &[M<'static>] = &[M::up(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/migrations/V1__initial.sql"
)))];
const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_ARRAY);

pub fn apply(connection: &mut Connection) -> Result<(), ChronicleError> {
    MIGRATIONS.to_latest(connection)?;
    Ok(())
}
