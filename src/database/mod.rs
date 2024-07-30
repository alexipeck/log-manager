pub mod model;

use diesel::{sqlite::Sqlite, Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::error::Error as StdError;
use tracing::error;

use crate::error::{DieselConnectionError, Error};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection(database_url: &str) -> Result<SqliteConnection, Error> {
    match SqliteConnection::establish(database_url) {
        Ok(connection) => Ok(connection),
        Err(err) => {
            error!("Error connecting to {database_url}. Err: {err}");
            Err(Error::DieselConnection(DieselConnectionError(err)))
        }
    }
}

pub fn run_migrations(
    connection: &mut impl MigrationHarness<Sqlite>,
    embedded_migrations: EmbeddedMigrations,
) -> Result<(), Box<dyn StdError + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(embedded_migrations)?;

    Ok(())
}
