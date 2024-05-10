use sqlx::{Pool, Sqlite};
use tracing::error;

use crate::constant::CACHE_SQLITE_DB;
use crate::database::sqlite::init::create_sqlite_file;
use crate::database::sqlite::migration::migration_dispatch::migrate_sqlite;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Initializes SQLite database.
///
/// This function checks if the SQLite database files exist and creates them if they don't.
/// It then initializes the database by creating the necessary tables and indices.
/// This function uses two separate SQLite databases: one for data and one for cache.
///
/// The function performs the following operations in order:
/// 1. Calls the `create_sqlite_file` function to create the SQLite database files for data and cache if they don't exist.
/// 2. Calls the `migrate_sqlite` function to perform any necessary migrations.
/// 3. Retrieves a connection pool to the SQLite cache database using the `get_sqlite_pool` function.
/// 4. Calls the `init_sqlite_cache` function to initialize the SQLite cache database.
/// 5. Closes the connection pool to the SQLite cache database.
/// 6. Retrieves a connection pool to the SQLite data database using the `get_sqlite_pool` function.
/// 7. Calls the `init_sqlite_data` function to initialize the SQLite data database.
/// 8. Closes the connection pool to the SQLite data database.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn init_sqlite() -> Result<(), AppError> {
    create_sqlite_file(CACHE_SQLITE_DB)?;
    if let Err(e) = migrate_sqlite().await {
        error!("{:?}", e);
    };
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    init_sqlite_cache(&pool).await?;
    pool.close().await;
    Ok(())
}

/// Initializes the SQLite cache database.
///
/// This function creates two tables in the SQLite cache database if they do not already exist:
/// 1. `request_cache` - This table stores cached responses to requests. It has three columns:
///     * `json` - The JSON representation of the request. This is the primary key of the table.
///     * `response` - The cached response to the request.
///     * `last_updated` - The timestamp of when the cached response was last updated.
/// 2. `cache_stats` - This table stores statistics about the cache. It has four columns:
///     * `key` - The key associated with the cache statistics. This is the primary key of the table.
///     * `response` - The cached response associated with the key.
///     * `last_updated` - The timestamp of when the cached response was last updated.
///     * `last_page` - The last page that was cached for the key.
///
/// # Arguments
///
/// * `pool` - A reference to the SQLite connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
async fn init_sqlite_cache(pool: &Pool<Sqlite>) -> Result<(), AppError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_cache (
            json TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to create the table. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache_stats (
            key TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL,
            last_page INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to create the table. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;
    Ok(())
}
