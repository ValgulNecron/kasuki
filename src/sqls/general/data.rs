use std::env;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{SqlInsertError, SqlSelectError};
use crate::sqls::sqlite::data::{
    get_data_guild_langage_sqlite, get_data_module_activation_status_sqlite,
    remove_data_activity_status_sqlite, set_data_activity_sqlite, set_data_guild_langage_sqlite,
    set_data_module_activation_status_sqlite, set_data_ping_history_sqlite,
};

pub async fn set_data_ping_history(shard_id: String, latency: String) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_ping_history_sqlite(shard_id, latency).await
    } else if db_type == *"postgresql" {
        Ok(())
    } else {
        set_data_ping_history_sqlite(shard_id, latency).await
    }
}

pub async fn get_data_guild_langage(
    guild_id: &str,
) -> Result<(Option<String>, Option<String>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_guild_langage_sqlite(guild_id).await
    } else if db_type == *"postgresql" {
        Ok((None, None))
    } else {
        get_data_guild_langage_sqlite(guild_id).await
    }
}

pub async fn set_data_guild_langage(guild_id: &String, lang: &String) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_guild_langage_sqlite(&guild_id, lang).await
    } else if db_type == *"postgresql" {
        Ok(())
    } else {
        set_data_guild_langage_sqlite(&guild_id, lang).await
    }
}

/*pub async fn get_data_activity() -> Vec<ActivityData> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_activity_sqlite().await
    } else if db_type == *"postgresql" {
        Vec::new()
    } else {
        get_data_activity_sqlite().await
    }
}*/

pub async fn set_data_activity(
    anime_id: i32,
    timestamp: i64,
    guild_id: String,
    webhook: String,
    episode: i32,
    name: String,
    delays: i64,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_activity_sqlite(
            anime_id, timestamp, guild_id, webhook, episode, name, delays,
        )
        .await
    } else if db_type == *"postgresql" {
        Ok(())
    } else {
        set_data_activity_sqlite(
            anime_id, timestamp, guild_id, webhook, episode, name, delays,
        )
        .await
    }
}

pub async fn get_data_module_activation_status(
    guild_id: &String,
) -> Result<(Option<String>, Option<bool>, Option<bool>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_module_activation_status_sqlite(guild_id).await
    } else if db_type == *"postgresql" {
        Err(SqlSelectError(String::from("Error")))
    } else {
        get_data_module_activation_status_sqlite(guild_id).await
    }
}

pub async fn set_data_module_activation_status(
    guild_id: &String,
    anilist_value: bool,
    ai_value: bool,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_module_activation_status_sqlite(guild_id, anilist_value, ai_value).await
    } else if db_type == *"postgresql" {
        Err(SqlInsertError(String::from("Error")))
    } else {
        set_data_module_activation_status_sqlite(guild_id, anilist_value, ai_value).await
    }
}

pub async fn remove_data_module_activation_status(
    server_id: String,
    anime_id: String,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        remove_data_activity_status_sqlite(server_id, anime_id).await
    } else if db_type == *"postgresql" {
        Err(SqlInsertError(String::from("Error")))
    } else {
        remove_data_activity_status_sqlite(server_id, anime_id).await
    }
}