use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;
use std::fs::File;
use std::path::Path;

/// Asynchronously establish a connection pool to a SQLite database.
///
/// # Arguments
///
/// * `database_url`: A string slice that holds the URL of the database.
///
/// # Returns
///
/// * `Pool<Sqlite>`: A pool of connections to the database.
///
/// # Panics
///
/// * This function will panic if it fails to establish a connection to the database.
///
/// # Examples
///
/// ```rust
/// let database_url = "sqlite:./my_db.db";
/// let pool = get_pool(database_url).await;
/// ```
///
/// # Notes
///
/// * This function is async and should be awaited.
///
pub async fn get_sqlite_pool(database_url: &str) -> Pool<Sqlite> {
    SqlitePool::connect(database_url).await.unwrap()
}

pub async fn init() {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == "sqlite".to_string() {
        init_sqlite().await
    } else if db_type == "postgresql".to_string() {
    } else {
        init_sqlite().await
    }
}

async fn init_sqlite() {
    let paths = ["./data.db", "./cache.db"];

    for path in &paths {
        let p = Path::new(path);
        if !p.exists() {
            match File::create(p) {
                Ok(_) => {
                    let pool = get_sqlite_pool(p.to_str().unwrap()).await;
                    init_sqlite_cache(&pool).await
                }
                Err(e) => {
                    println!("Failed to create the file {} : {}", path, e);
                    return;
                }
            }
        }
    }
}

async fn init_sqlite_cache(pool: &Pool<Sqlite>) {}
