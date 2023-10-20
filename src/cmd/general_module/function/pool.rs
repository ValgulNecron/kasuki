use sqlx::{Pool, Sqlite, SqlitePool};

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
pub async fn get_pool(database_url: &str) -> Pool<Sqlite> {
    SqlitePool::connect(database_url).await.unwrap()
}
