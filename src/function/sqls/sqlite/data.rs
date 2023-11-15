use crate::constant::DATA_SQLITE_DB;
use crate::function::sqls::sqlite::pool::get_sqlite_pool;
use chrono::Utc;
use log::error;

pub async fn set_data_ping_history_sqlite(shard_id: String, latency: String) {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await;
    let now = Utc::now().timestamp().to_string();
    match sqlx::query(
        "INSERT OR REPLACE INTO ping_history (shard_id, timestamp, ping) VALUES (?, ?, ?)",
    )
    .bind(shard_id)
    .bind(now)
    .bind(latency)
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            error!("Error while creating the table: {}", e)
        }
    };
}
