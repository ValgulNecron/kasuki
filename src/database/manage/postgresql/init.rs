use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use crate::config::BotConfigDetails;
use crate::database::manage::postgresql::migration::migration_dispatch::migrate_postgres;
use crate::database::manage::postgresql::pool::{get_postgresql_pool, get_postgresql_pool_without_db};
use crate::helper::error_management::error_enum;
use sqlx::{Pool, Postgres};
use tracing::{error, trace, warn};

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
pub async fn init_postgres(db_config: BotConfigDetails) -> Result<(), Box<dyn Error>> {
    create_and_complete_dot_pgpass(db_config.clone())?;
    match migrate_postgres(db_config.clone()).await {
        Ok(_) => {}
        Err(e) => {
            let e = e.to_string().replace("\\\\n", "\n");
            warn!("Failed to migrate the PostgreSQL database: {}", e);
        }
    };
    let pool = get_postgresql_pool_without_db(db_config.clone()).await?;
    init_postgres_data(&pool,db_config).await?;
    pool.close().await;
    Ok(())
}

use dirs;
use url::quirks::port;
use crate::helper::error_management::error_enum::Error::Byte;
use crate::helper::error_management::error_enum::UnknownResponseError;

fn create_and_complete_dot_pgpass(db_config: BotConfigDetails) -> Result<(), Box<dyn Error>> {
    let mut home_dir = dirs::home_dir().ok_or(Box::new(UnknownResponseError::Option(String::from("Failed to get home directory."))))?;
    #[cfg(windows)]
    {
        home_dir = home_dir.join("AppData\\Roaming\\postgres\\")
    }
    #[cfg(not(windows))]
    {
        home_dir = home_dir.join("postgres/")
    }
    #[cfg(windows)]
    let pgpass_path = home_dir.join("pgpass.conf");
    #[cfg(not(windows))]
    let pgpass_path = home_dir.join(".pgpass");
    trace!(pgpass_path = ?pgpass_path);
    trace!(home_dir = ?home_dir);

    fs::create_dir_all(home_dir)?;
    let mut open = OpenOptions::new()
        .write(true)
        .create_new(true)
        .read(true)
        .open(&pgpass_path)?;
    let mut cursor: Vec<u8> = Vec::new();
    let data = open.read(&mut cursor)?;
    let pg_path_data = format!("{}:{}:kasuki:{}:{}", db_config.host.clone().unwrap_or_default(), db_config.port.clone().unwrap_or_default()
                               ,db_config.user.clone().unwrap_or_default(), db_config.password.clone().unwrap_or_default());
    let pg_path_data_chars: &[char] = &pg_path_data.chars().collect::<Vec<char>>();
    if data.to_string().contains(pg_path_data_chars) {
        return Ok(());
    }

    open.write_all(pg_path_data.as_bytes())?;
    open.flush()?;
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
async fn init_postgres_data(pool: &Pool<Postgres>,db_config: BotConfigDetails) -> Result<(), Box<dyn Error>> {
    // If the database does not exist, create it
    match sqlx::query("CREATE DATABASE kasuki")
        .execute(pool)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            warn!("Failed to create the kasuki database: {:#?}", e);
        }
    }
    let pool = &get_postgresql_pool(db_config).await?;
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
        error_enum::Error::Database(format!("Failed to create the table. {:#?}", e))
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS guild_lang (
            guild TEXT PRIMARY KEY,
            lang TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

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
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_activation (
        guild_id TEXT PRIMARY KEY,
        ai_module BIGINT NOT NULL,
        anilist_module BIGINT NOT NULL,
        game_module BIGINT NOT NULL,
        new_member BIGINT NOT NULL,
        anime BIGINT NOT NULL,
        vn BIGINT NOT NULL
   )",
    )
    .execute(pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS registered_user  (
            user_id TEXT PRIMARY KEY,
            anilist_id TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS global_kill_switch (
            guild_id TEXT PRIMARY KEY,
            ai_module BIGINT NOT NULL,
            anilist_module BIGINT NOT NULL,
            game_module BIGINT NOT NULL,
            new_member BIGINT NOT NULL,
            anime BIGINT NOT NULL,
            vn BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

    // on conflict do nothing
    sqlx::query(
        "INSERT INTO global_kill_switch
        (guild_id, anilist_module, ai_module, game_module, new_member, anime, vn)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (guild_id) DO NOTHING",
    )
    .bind("1")
    .bind(1)
    .bind(1)
    .bind(1)
    .bind(1)
    .bind(1)
    .bind(1)
    .execute(pool)
    .await
    .map_err(|e| {
        error_enum::Error::Database(format!("Failed to insert into the table. {:#?}", e))
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
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

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
    .map_err(|e| error_enum::Error::Database(format!("Failed to create the table. {:#?}", e)))?;

    Ok(())
}
