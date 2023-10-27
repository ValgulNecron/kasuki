use crate::function::sql::sqlite::pool::get_sqlite_pool;
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
    let pool = get_sqlite_pool(paths[0]).await;
    init_sqlite_data(&pool).await;
}

async fn init_sqlite_cache(pool: &Pool<Sqlite>) {}

async fn init_sqlite_data(pool: &Pool<Sqlite>) {}
