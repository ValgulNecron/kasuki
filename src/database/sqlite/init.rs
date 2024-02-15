use std::fs::File;
use std::path::Path;

use sqlx::{Pool, Sqlite};
use tracing::error;

use crate::constant::{CACHE_SQLITE_DB, DATA_SQLITE_DB};
use crate::database::sqlite::migration::migration_dispatch::migrate_sqlite;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NotACommandError;
use crate::error_enum::NotACommandError::{
    CreatingDatabaseFileError, CreatingTableError, InsertingDatabaseError,
};

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
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
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
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
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
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
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
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS activity_data (
        anime_id TEXT,
        timestamp TEXT,
        server_id TEXT,
        webhook TEXT,
        episode TEXT,
        name TEXT,
        delays INTEGER DEFAULT 0,
        image TEXT DEFAULT 'data:image/png;base64,UklGRngFAABXRUJQVlA4IGwFAACQFQCdASpAAEAAPm0qkUWkIqGWDVeYQAbEoAvNeYDiDjvgYVI5A1hflD+HyFjgvqtG2WTHL+FrSd/NOhgzpPWXsEeWd7MPRV/cB30UGJsvLnbV1VqthWilezYCFAvetUFlshCk/xVa9IBRnL1Pp8HXWSJoXAGlB/F+74dS9N1/WmAgqV2g74G6aecKudBqp5vcYaiypIM+1wJSN29GOOiwmRrbfLgukEaogfmrmNBdDsfe9sQAAP7/toyQcQm08zUVT/Gcrfn+ngVdDMzXP4jQ9Hm0cRUTMClcf86TQ39SGSgXPW2HMB6sMZ2Vsv0/GOWT/WU+J+IfOI0Ai0aBHCQxDOZFaxrrdkA58PFTCDV9vJuUJkNTMqKBBoqRkica7jt1zj2Z23YDDhpCnNm00qBio5nLkVV0u83H3Qnbaz9kP3xd2llOCqg27pmLBEhtd3QJlmfoK+JMuIEhJgqOB5I1fXsnzBWQiz2mOvgSiwJAuuqAldkH4vt3085f8eCgoHu199XFVbU332orUfhK+G2vn7TQd/7VpAb0lxQDDyT00Zlxxy+O3/5IfIWIzJ1CTofiw6CL6Eew9xeAmtUoXAEW/ItApLeina9bCTKqiOfEVpnMHOBP+Rb9KMV1ugp6W4uYQ1/ontVdrBGxlXgbMI5otyujVdyUXd4P/PuS5i0Zkw6htI7E5sPsPowT5WAuADForLZb8lLSv8qGX4u4T+Pf4/qyH2nRzBg1CMokQIwSyeO27e5csfrjyx1x0AGcC/uB2N43gdNr/wAiud3isIiQVKYhBuckmRXUJZsILOKVPOdO37meUFDoNDGB9fJoOCcEvwYNIebfA/5rlXItCzE4ah0kXDWP5GWbpq7dNyE+6GFbo8IHcg5IEgvtwM3J54BYBkADUJD9C+VHBAPzEQnv0e3yW0q6xgMDawUKfPHaY5vXK/uShArGmWwvhZwXMYtWnB+j2QiRMVC2PY/jlZYuEjafnGcFZdelBZWXGd8ewCyUVBl/LZYjEhcs7mGCj4VtsfP6onACpogMRlr8j302Jdt/Gf+XqbC+3wUmLVgKvrmWwve+yMEeI7IsmR1xLMZlGPmfvoohtoMYp3J3ogXMa66etFb2L3L+y3pvb1rh08nXxNS40gEYXWNb2+3Dcr4iHiAuP8FkMF0snZEPOz/3EOn0IGnmqQ7S34g48tTUJylINg3uXOeRZgHLncuThPa1Igb+ZdZ79ndbQ0LBUemYhtbD9KKXPDhnFvxMwOBPgOyB/rjrJ5eJ9/O8V5EhKBRtcAaHDBiyDUVGIz7EKZ/FT39pP/XtwKiVnDmd69AoGHBeeyhwgDV7rVbwB0lQW3xwddCfiA/vGqWypJ6l6c0DfPSw3dgQv5bZMKLWjQyD6EZBPDOCi2p0iQsgD7wlIxZylSxYJlfNxJbw9l/PT0mQk9CdNwQYUvaXKbOrSYjfNpkFE6Tae6gURO2X3M+sPV0UQr3IuiD06ur5wGUQrXK/Ldl+JopFMeQb/8wvMgcKiwmrmQf6MaWcZnCb4CFbf1xAxG2KK2m/H47PHg8RYJH/X+8fb6hqBLq7DWKxUlAuB25UL0MA+oXZwahI8s4AeTjm65G8m5PS2Z+VBcYwsUHDK6pbcFMul9CbIQBtX/HLHetzai1n+MielaSfiY78EY8ZMAuZXYD/xC/b0NZyQDQ76fOokEUdTzU5ev2QdJMtI1UEtE+6toA8jPzy568UDnSvfZLCr2Rj4IP5bXMUhTCumOalI/4kAGmughKGrp+dm1MPN3Tg/oJPXy//yNYw9bton7goPbN5nyg0Yjfyr4Qq6V+twPgYTbRIC+fHeH0/agtR3nWQYE8PEAAAAA==',
        PRIMARY KEY (anime_id, server_id)
    )",
    )
        .execute(pool)
        .await
        .map_err(|e| NotACommandError(CreatingTableError(format!("Failed to create the table. {}", e))))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_activation (
       guild_id TEXT PRIMARY KEY,
       ai_module INTEGER,
       anilist_module INTEGER,
        game_module INTEGER,
            new_member INTEGER
   )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS registered_user  (
            user_id TEXT PRIMARY KEY,
            anilist_id TEXT
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS global_kill_switch (
            id TEXT PRIMARY KEY,
            ai_module INTEGER,
            anilist_module INTEGER,
            game_module INTEGER,
            new_member INTEGER
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
    })?;

    sqlx::query(
        "INSERT OR REPLACE INTO global_kill_switch (id, anilist_module, ai_module, game_module, new_member) VALUES (?, ?, ?, ?, ?)",
    )
        .bind("1")
        .bind(1)
        .bind(1)
        .bind(1)
        .bind(1)
        .execute(pool)
        .await.map_err(|e| NotACommandError(InsertingDatabaseError(format!("Failed to create the database table. {}", e))))?;

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
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS server_image (
                server_id TEXT,
                type TEXT,
                image TEXT NOT NULL,
        PRIMARY KEY (server_id, type)

     )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        NotACommandError(CreatingTableError(format!(
            "Failed to create the table. {}",
            e
        )))
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
                return Err(NotACommandError(CreatingDatabaseFileError(format!(
                    "Failed to create db file. {}",
                    e
                ))));
            }
        }
    }
    Ok(())
}
