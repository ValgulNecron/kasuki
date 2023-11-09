use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn get_sqlite_pool(database_url: &str) -> Pool<Sqlite> {
    SqlitePool::connect(database_url).await.unwrap()
}
