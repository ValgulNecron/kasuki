use std::fs::File;
use std::path::Path;

use sqlx::{Pool, Sqlite};
use tracing::error;

use crate::constant::{CACHE_SQLITE_DB, DATA_SQLITE_DB};
use crate::database::manage::sqlite::migration::migration_dispatch::migrate_sqlite;
use crate::database::manage::sqlite::pool::get_sqlite_pool;
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
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    init_sqlite_data(&pool).await?;
    pool.close().await;
    Ok(())
}

/// Initializes the SQLite data database.
///
/// This function creates several tables in the SQLite data database if they do not already exist:
/// 1. `ping_history` - This table stores the history of pings. It has three columns:
///     * `shard_id` - The ID of the shard. This, along with `timestamp`, forms the primary key of the table.
///     * `timestamp` - The timestamp of the ping.
///     * `ping` - The ping value.
/// 2. `guild_lang` - This table stores the language settings for each guild. It has two columns:
///     * `guild` - The ID of the guild. This is the primary key of the table.
///     * `lang` - The language setting for the guild.
/// 3. `activity_data` - This table stores activity data. It has eight columns:
///     * `anime_id` - The ID of the anime.
///     * `server_id` - The ID of the server.
///     * `timestamp` - The timestamp of the activity.
///     * `webhook` - The webhook associated with the activity.
///     * `episode` - The episode associated with the activity.
///     * `name` - The name associated with the activity.
///     * `delays` - The number of delays associated with the activity.
///     * `image` - The image associated with the activity.
/// 4. `module_activation` - This table stores module activation settings for each guild. It has six columns:
///     * `guild_id` - The ID of the guild. This is the primary key of the table.
///     * `ai_module` - The activation setting for the AI module.
///     * `anilist_module` - The activation setting for the Anilist module.
///     * `game_module` - The activation setting for the game module.
///     * `new_member` - The activation setting for the new member module.
///     * `anime` - The activation setting for the anime module.
/// 5. `registered_user` - This table stores registered users. It has two columns:
///     * `user_id` - The ID of the user. This is the primary key of the table.
///     * `anilist_id` - The Anilist ID of the user.
/// 6. `global_kill_switch` - This table stores the global kill switch settings. It has five columns:
///     * `id` - The ID of the setting. This is the primary key of the table.
///     * `ai_module` - The global kill switch setting for the AI module.
///     * `anilist_module` - The global kill switch setting for the Anilist module.
///     * `game_module` - The global kill switch setting for the game module.
///     * `new_member` - The global kill switch setting for the new member module.
/// 7. `user_color` - This table stores user color settings. It has four columns:
///     * `user_id` - The ID of the user. This is the primary key of the table.
///     * `color` - The color setting for the user.
///     * `pfp_url` - The profile picture URL for the user.
///     * `image` - The image for the user.
/// 8. `server_image` - This table stores server image settings. It has four columns:
///     * `server_id` - The ID of the server.
///     * `type` - The type of the image.
///     * `image` - The image.
///     * `image_url` - The URL of the image.
///
/// # Arguments
///
/// * `pool` - A reference to the SQLite connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
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

/// Creates a SQLite database file at the specified path if it does not already exist.
///
/// This function performs the following operations in order:
/// 1. Creates a new `Path` instance from the provided string.
/// 2. Checks if a file already exists at the path.
/// 3. If a file does not exist, attempts to create a new file at the path.
/// 4. If the file creation fails, logs an error and returns an `AppError`.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path where the SQLite database file should be created.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub fn create_sqlite_file(path: &str) -> Result<(), AppError> {
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
