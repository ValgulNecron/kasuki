use sqlx::{Pool, Sqlite, SqlitePool};

use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Connects to a SQLite database and returns a connection pool.
/// The function is asynchronous and returns a `Pool` wrapped in a `Result`.
///
/// # Arguments
///
/// * `database_url` - A string slice representing the URL to the SQLite database.
///
/// # Returns
///
/// The function returns a `Pool<Sqlite>` wrapped in a `Result`. If the connection to
/// the database is successful, the `Pool` is returned. Otherwise, an error is returned.
///
pub async fn get_sqlite_pool(database_url: &str) -> Result<Pool<Sqlite>, AppError> {
    SqlitePool::connect(database_url).await.map_err(|e| {
        AppError::new(
            format!("Failed to get the sqlite pool: {}", e),
            ErrorType::Database,
            ErrorResponseType::Unknown,
        )
    })
}
