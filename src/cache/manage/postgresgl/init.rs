use sqlx::{Pool, Postgres};

use crate::database::postgresql::migration::migration_dispatch::migrate_postgres;
use crate::database::postgresql::pool::get_postgresql_pool;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Initializes the PostgreSQL database.
///
/// This function performs the following operations in order:
/// 1. Calls the `migrate_postgres` function to migrate the PostgreSQL database.
/// 2. Retrieves a connection pool to the PostgreSQL database using the `get_postgresql_pool` function.
/// 3. Calls the `init_postgres_cache` function to initialize the cache in the PostgreSQL database.
/// 4. Closes the connection pool.
/// 5. Retrieves a new connection pool to the PostgreSQL database using the `get_postgresql_pool` function.
/// 6. Calls the `init_postgres_data` function to initialize the data in the PostgreSQL database.
/// 7. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn init_postgres() -> Result<(), AppError> {
    migrate_postgres().await?;
    let pool = get_postgresql_pool().await?;
    init_postgres_cache(&pool).await?;
    pool.close().await;
    Ok(())
}

/// Initializes the cache in the PostgreSQL database.
///
/// This function performs the following operations in order:
/// 1. Checks if the `CACHE` database exists.
/// 2. If the `CACHE` database does not exist, it creates it.
/// 3. Creates the `request_cache` table if it does not exist. This table has the following fields:
///    * `json`: A TEXT field that is the primary key.
///    * `response`: A TEXT field that is not nullable.
///    * `last_updated`: A BIGINT field that is not nullable.
/// 4. Creates the `cache_stats` table if it does not exist. This table has the following fields:
///    * `key`: A TEXT field that is the primary key.
///    * `response`: A TEXT field that is not nullable.
///    * `last_updated`: A BIGINT field that is not nullable.
///    * `last_page`: A BIGINT field that is not nullable.
///
/// # Parameters
///
/// * `pool`: A `Pool<Postgres>` reference that represents the connection pool to the PostgreSQL database.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
async fn init_postgres_cache(pool: &Pool<Postgres>) -> Result<(), AppError> {
    // Check if the database exists
    let exists: (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
            .bind("CACHE")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to check if the database exists. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;

    // If the database does not exist, create it
    if !exists.0 {
        sqlx::query("CREATE DATABASE CACHE")
            .execute(pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to create the database. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_cache (
           json TEXT PRIMARY KEY,
           response TEXT NOT NULL,
           last_updated BIGINT NOT NULL
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
           last_updated BIGINT NOT NULL,
           last_page BIGINT NOT NULL
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