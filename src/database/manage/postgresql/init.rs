use sqlx::{Pool, Postgres};

use crate::database::manage::postgresql::migration::migration_dispatch::migrate_postgres;
use crate::database::manage::postgresql::pool::get_postgresql_pool;
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
    init_postgres_data(&pool).await?;
    pool.close().await;
    Ok(())
}

/// Initializes the data in the PostgreSQL database.
///
/// This function performs the following operations in order:
/// 1. Checks if the `DATA` database exists.
/// 2. If the `DATA` database does not exist, it creates it.
/// 3. Creates the `ping_history`, `guild_lang`, `activity_data`, `module_activation`, `registered_user`, `global_kill_switch`, `user_color`, and `server_image` tables if they do not exist.
/// 4. Inserts default values into the `global_kill_switch` table if they do not exist.
///
/// # Parameters
///
/// * `pool`: A `Pool<Postgres>` reference that represents the connection pool to the PostgreSQL database.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
async fn init_postgres_data(pool: &Pool<Postgres>) -> Result<(), AppError> {
    // Check if the database exists
    let exists: (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
            .bind("DATA")
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
        sqlx::query("CREATE DATABASE DATA")
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
        "CREATE TABLE IF NOT EXISTS ping_history (
                    shard_id TEXT,
                    timestamp TEXT,
                    ping TEXT NOT NULL,
                    PRIMARY KEY (shard_id, timestamp)
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
        "CREATE TABLE IF NOT EXISTS guild_lang (
            guild TEXT PRIMARY KEY,
            lang TEXT NOT NULL
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
        "CREATE TABLE IF NOT EXISTS activity_data (
        anime_id TEXT,
        server_id TEXT,
        timestamp TEXT NOT NULL,
        webhook TEXT NOT NULL,
        episode TEXT NOT NULL,
        name TEXT NOT NULL,
        delays BIGINT DEFAULT 0,
        image TEXT NOT NULL,
        PRIMARY KEY (anime_id, server_id)
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
        "CREATE TABLE IF NOT EXISTS module_activation (
        guild_id TEXT PRIMARY KEY,
        ai_module BIGINT NOT NULL,
        anilist_module BIGINT NOT NULL,
        game_module BIGINT NOT NULL,
        new_member BIGINT NOT NULL,
        anime BIGINT NOT NULL
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
        "CREATE TABLE IF NOT EXISTS registered_user  (
            user_id TEXT PRIMARY KEY,
            anilist_id TEXT NOT NULL
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
        "CREATE TABLE IF NOT EXISTS global_kill_switch (
            id TEXT PRIMARY KEY,
            ai_module BIGINT NOT NULL,
            anilist_module BIGINT NOT NULL,
            game_module BIGINT NOT NULL,
            new_member BIGINT NOT NULL
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
        "INSERT INTO global_kill_switch
        (id, anilist_module, ai_module, game_module, new_member)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id)
            DO UPDATE
            SET anilist_module = excluded.anilist_module,
            ai_module = excluded.ai_module,
            game_module = excluded.game_module,
            new_member = excluded.new_member",
    )
    .bind("1")
    .bind(1)
    .bind(1)
    .bind(1)
    .bind(1)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to insert into the table. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_color (
            user_id TEXT PRIMARY KEY,
            color TEXT NOT NULL,
            pfp_url TEXT NOT NULL,
            image TEXT NOT NULL
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
        "CREATE TABLE IF NOT EXISTS server_image (
            server_id TEXT PRIMARY KEY,
            type TEXT PRIMARY KEY,
            image TEXT NOT NULL,
            image_url TEXT NOT NULL
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
