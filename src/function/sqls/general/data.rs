use crate::function::sqls::sqlite::data::{
    get_data_guild_langage_sqlite, set_data_activity_sqlite, set_data_guild_langage_sqlite,
    set_data_ping_history_sqlite,
};
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

pub async fn get_data_guild_langage(guild_id: &str) -> (Option<String>, Option<String>) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_guild_langage_sqlite(guild_id).await
    } else if db_type == *"postgresql" {
        (None, None)
    } else {
        get_data_guild_langage_sqlite(guild_id).await
    }
}

pub async fn set_data_guild_langage(guild_id: String, lang: &String) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_guild_langage_sqlite(guild_id, lang).await
    } else if db_type == *"postgresql" {
    } else {
        set_data_guild_langage_sqlite(guild_id, lang).await
    }
}

pub async fn get_data_activity() {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_guild_langage_sqlite().await
    } else if db_type == *"postgresql" {
    } else {
        set_data_guild_langage_sqlite().await
    }
}

pub async fn set_data_activity(
    anime_id: i32,
    timestamp: i64,
    guild_id: String,
    webhook: String,
    episode: i32,
    name: String,
    delays: i64,
) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_activity_sqlite(
            anime_id, timestamp, guild_id, webhook, episode, name, delays,
        )
        .await
    } else if db_type == *"postgresql" {
    } else {
        set_data_activity_sqlite(
            anime_id, timestamp, guild_id, webhook, episode, name, delays,
        )
        .await
    }
}
