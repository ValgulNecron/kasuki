use crate::function::sqls::sqlite::data::set_data_ping_history_sqlite;
use std::env;

pub async fn set_data_ping_history(shard_id: String, latency: String) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_ping_history_sqlite(shard_id, latency).await
    } else if db_type == *"postgresql" {
    } else {
        set_data_ping_history_sqlite(shard_id, latency).await
    }
}
