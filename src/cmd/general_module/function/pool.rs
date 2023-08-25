use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn get_pool(database_url: &str) -> Pool<Sqlite> {
    let pool = SqlitePool::connect(&database_url).await.unwrap();
    pool
}
