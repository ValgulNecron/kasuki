use crate::constant::{CACHE_SQLITE_DB, DATA_SQLITE_DB};
use crate::function::sqls::sqlite::pool::get_sqlite_pool;
use log::error;
use sqlx::{Pool, Sqlite};
use std::fs::File;
use std::path::Path;

pub async fn init_sqlite() {
    let p = Path::new(DATA_SQLITE_DB);
    if !p.exists() {
        match File::create(p) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to create the file {} : {}", DATA_SQLITE_DB, e);
                return;
            }
        }
    }
    let p = Path::new(CACHE_SQLITE_DB);
    if !p.exists() {
        match File::create(p) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to create the file {} : {}", CACHE_SQLITE_DB, e);
                return;
            }
        }
    }
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    init_sqlite_cache(&pool).await;
    pool.close().await;
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    init_sqlite_data(&pool).await;
    pool.close().await;
}

async fn init_sqlite_cache(pool: &Pool<Sqlite>) {
    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_cache (
            json TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache_stats (
            key TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL,
            last_page INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
}

async fn init_sqlite_data(_pool: &Pool<Sqlite>) {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await;

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS ping_history (
                    shard_id TEXT,
                    timestamp TEXT,
                    ping TEXT NOT NULL,
                    PRIMARY KEY (shard_id, timestamp)
                )",
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            error!("Error while creating the table: {}", e)
        }
    };

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS guild_lang (
            guild TEXT PRIMARY KEY,
            lang TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
}
