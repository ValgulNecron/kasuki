use crate::function::sqls::sqlite::pool::get_sqlite_pool;
use log::error;
use sqlx::{Pool, Sqlite};
use std::fs::File;
use std::path::Path;

pub async fn init_sqlite() {
    let paths = ["./data.db", "./cache.db"];

    for path in &paths {
        let p = Path::new(path);
        if !p.exists() {
            match File::create(p) {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to create the file {} : {}", path, e);
                    return;
                }
            }
        }
    }
    let pool = get_sqlite_pool(paths[1]).await;
    init_sqlite_cache(&pool).await;
    pool.close().await;
    let pool = get_sqlite_pool(paths[0]).await;
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

async fn init_sqlite_data(_pool: &Pool<Sqlite>) {}
