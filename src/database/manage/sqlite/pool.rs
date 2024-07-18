use std::error::Error;

use sqlx::{Pool, Sqlite, SqlitePool};

use crate::helper::error_management::error_enum::UnknownResponseError;

/// Establishes a connection to a SQLite database and returns a connection pool.
///
/// This function is asynchronous and returns a `Pool` object wrapped in a `Result`. The `Pool` object represents a pool of database connections.
///
/// # Arguments
///
/// * `database_url` - A string slice that holds the URL to the SQLite database. This URL specifies the location of the database file.
///
/// # Returns
///
/// * A `Result` that is either:
///     * An `Ok` variant containing a `Pool<Sqlite>` object if the connection to the database was successful. This `Pool` object can be used to interact with the database.
///     * An `Err` variant containing an `AppError` object if the connection to the database failed. This `AppError` object contains details about the error.
///
/// # Errors
///
/// This function will return an error if the connection to the database fails. The error will be of type `AppError`, with the `ErrorType` set to `Database` and the `ErrorResponseType` set to `Unknown`.
pub async fn get_sqlite_pool(database_url: &str) -> Result<Pool<Sqlite>, Box<dyn Error>> {
    let pool = SqlitePool::connect(database_url).await.map_err(|e| {
        UnknownResponseError::Database(format!("Failed to connect to the sqlite pool: {:#?}", e))
    })?;
    Ok(pool)
}
