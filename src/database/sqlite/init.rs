use std::fs::File;
use std::path::Path;

use sqlx::{Pool, Sqlite};
use tracing::error;

use crate::constant::{CACHE_SQLITE_DB, DATA_SQLITE_DB};
use crate::database::sqlite::migration::migration_dispatch::migrate_sqlite;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Initializes SQLite database.
///
/// This function checks if the SQLite database files exist and creates them if they don't.
/// It then initializes the database by creating the necessary tables and indices.
/// This function uses two separate SQLite databases: one for data and one for cache.
pub async fn init_sqlite() -> Result<(), AppError> {
    create_sqlite_file(DATA_SQLITE_DB)?;
    create_sqlite_file(CACHE_SQLITE_DB)?;
    if let Err(e) = migrate_sqlite().await {
        error!("{:?}", e);
    };
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    init_sqlite_cache(&pool).await?;
    pool.close().await;
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    init_sqlite_data(&pool).await?;
    pool.close().await;
    Ok(())
}

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

/// Initializes the SQLite tables and data.
///
/// # Arguments
///
/// * `_pool` - A reference to the SQLite connection pool.
async fn init_sqlite_data(pool: &Pool<Sqlite>) -> Result<(), AppError> {
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
        delays INTEGER DEFAULT 0,
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
            ai_module INTEGER NOT NULL,
            anilist_module INTEGER NOT NULL,
            game_module INTEGER NOT NULL,
            new_member INTEGER NOT NULL,
            anime INTEGER NOT NULL
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
            ai_module INTEGER NOT NULL,
            anilist_module INTEGER NOT NULL,
            game_module INTEGER NOT NULL,
            new_member INTEGER NOT NULL
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
        "INSERT OR REPLACE INTO global_kill_switch
        (id, anilist_module, ai_module, game_module, new_member)
        VALUES (?, ?, ?, ?, ?)",
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
                server_id TEXT,
                type TEXT,
                image TEXT NOT NULL,
                image_url TEXT NOT NULL,
                PRIMARY KEY (server_id, type)
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

fn create_sqlite_file(path: &str) -> Result<(), AppError> {
    let p = Path::new(path);
    if !p.exists() {
        match File::create(p) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create the file {} : {}", path, e);
                return Err(AppError::new(
                    format!("Failed to create db file. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                ));
            }
        }
    }
    Ok(())
}
