use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

use crate::errors::ChronicleError;

const MIGRATION_ARRAY: &[M<'static>] = &[
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V1__initial.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V2__milestone_1.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V3__milestone_2.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V4__milestone_3_watchers.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V5__milestone_4_identity.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V6__milestone_5_version_families.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V7__milestone_6_projects_and_sessions.sql"
    ))),
    M::up(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/V8__milestone_7_content_indexing.sql"
    ))),
];
const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_ARRAY);

pub fn apply(connection: &mut Connection) -> Result<(), ChronicleError> {
    MIGRATIONS.to_latest(connection)?;
    Ok(())
}
