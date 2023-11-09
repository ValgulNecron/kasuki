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

pub async fn init() {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        init_sqlite().await
    } else if db_type == *"postgresql" {
    } else {
        init_sqlite().await
    }
}
