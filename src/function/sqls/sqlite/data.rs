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

pub async fn get_data_guild_langage_sqlite(guild_id: &str) -> (Option<String>, Option<String>) {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT lang, guild FROM guild_lang WHERE guild = ?")
            .bind(guild_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    row
}

pub async fn set_data_guild_langage_sqlite(guild_id: String, lang: &String) {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await;
    match sqlx::query("INSERT OR REPLACE INTO guild_lang (guild, lang) VALUES (?, ?)")
        .bind(guild_id)
        .bind(lang)
        .execute(&pool)
        .await
    {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
}

pub async fn get_data_activity_sqlite() {}

pub async fn set_data_activity_sqlite(
    anime_id: i32,
    timestamp: i64,
    guild_id: String,
    webhook: String,
    episode: i32,
    name: String,
    delays: i64,
) {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await;
    match sqlx::query(
        "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook, episode, name, delays) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(anime_id)
        .bind(timestamp)
        .bind(guild_id)
        .bind(webhook)
        .bind(episode)
        .bind(name)
        .bind(delays)
        .execute(&pool)
        .await
    {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
}
