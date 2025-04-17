
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use crate::error::{Error, Result};

pub mod models;
pub mod participation_ops;
pub mod collab_ops;
pub mod csconfig_ops;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection(database_url: &str) -> Result<PgConnection> {
    Ok(PgConnection::establish(database_url)?)
}

pub fn run_pending_migrations(connection: &mut PgConnection) -> Result<()> {
    connection.run_pending_migrations(MIGRATIONS).map_err(|_| {
        Error::from("Unable to run migrations on database.")
    })?;
    Ok(())
}