use crate::function::sqls::sqlite::cache::get_random_cache_sqlite;
use crate::function::sqls::sqlite::init::init_sqlite;
use std::env;

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

pub async fn init_sql() {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        init_sqlite().await
    } else if db_type == *"postgresql" {
    } else {
        init_sqlite().await
    }
}

pub async fn get_random_cache(random_type: &str) -> (Option<String>, Option<i64>, Option<i64>) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_random_cache_sqlite(random_type).await
    } else if db_type == *"postgresql" {
        (None, None, None)
    } else {
        get_random_cache_sqlite(random_type).await
    }
}
