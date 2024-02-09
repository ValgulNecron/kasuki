use crate::anilist_struct::run::minimal_anime::ActivityData;
use crate::database::postgresql::pool::get_postgresql_pool;
use crate::database_struct::server_activity_struct::{ServerActivity, ServerActivityFull};
use crate::database_struct::user_color_struct::UserColor;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::CommandError::SqlInsertError;
use chrono::Utc;

pub async fn set_data_ping_history_postgresql(
    shard_id: String,
    latency: String,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    let now = Utc::now().timestamp().to_string();
    sqlx::query(
        "INSERT INTO DATA.ping_history (shard_id, timestamp, ping) VALUES ($1, $2, $3) ON CONFLICT (shard_id) DO UPDATE SET timestamp = EXCLUDED.timestamp, ping = EXCLUDED.ping",
    )
        .bind(shard_id)
        .bind(now)
        .bind(latency)
        .execute(&pool)
        .await
        .map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;
    Ok(())
}

pub async fn get_data_guild_language_postgresql(
    guild_id: String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let pool = get_postgresql_pool().await?;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT lang, guild FROM DATA.guild_lang WHERE guild = $1")
            .bind(guild_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    pool.close().await;
    Ok(row)
}

pub async fn set_data_guild_language_postgresql(
    guild_id: &String,
    lang: &String,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query(
        "INSERT INTO DATA.guild_lang (guild, lang) VALUES ($1, $2) ON CONFLICT (guild) DO UPDATE SET lang = EXCLUDED.lang",
    )
        .bind(guild_id)
        .bind(lang)
        .execute(&pool)
        .await
        .map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;
    Ok(())
}

pub async fn get_data_activity_postgresql(now: String) -> Result<Vec<ActivityData>, AppError> {
    let pool = get_postgresql_pool().await?;
    let rows: Vec<ActivityData> = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook, episode, name, delays, image FROM DATA.activity_data WHERE timestamp = $1",
    )
        .bind(now)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Ok(rows)
}

pub async fn set_data_activity_postgresql(
    server_activity_full: ServerActivityFull,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query(
        "INSERT INTO DATA.activity_data (anime_id, timestamp, server_id, webhook, episode, name, delays, image) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (anime_id) DO UPDATE SET timestamp = EXCLUDED.timestamp, server_id = EXCLUDED.server_id, webhook = EXCLUDED.webhook, episode = EXCLUDED.episode, name = EXCLUDED.name, delays = EXCLUDED.delays",
    )
        .bind(server_activity_full.anime_id)
        .bind(server_activity_full.timestamp)
        .bind(server_activity_full.guild_id)
        .bind(server_activity_full.webhook)
        .bind(server_activity_full.episode)
        .bind(server_activity_full.name)
        .bind(server_activity_full.delays)
        .bind(server_activity_full.image)
        .execute(&pool)
        .await.map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;
    Ok(())
}

pub async fn get_data_module_activation_status_postgresql(
    guild_id: &String,
) -> Result<(Option<String>, Option<bool>, Option<bool>, Option<bool>, Option<bool>), AppError> {
    let pool = get_postgresql_pool().await?;
    let row: (Option<String>, Option<bool>, Option<bool>, Option<bool>, Option<bool>) = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module, game_module, new_member FROM DATA.module_activation WHERE guild = $1",
    )
        .bind(guild_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None, None, None));
    pool.close().await;
    Ok(row)
}

pub async fn set_data_module_activation_status_postgresql(
    guild_id: &String,
    anilist_value: bool,
    ai_value: bool,
    game_value: bool,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query(
        "INSERT INTO DATA.module_activation (guild_id, anilist_module, ai_module, game_module) VALUES ($1, $2, $3, $4) ON CONFLICT (guild_id) DO UPDATE SET anilist_module = EXCLUDED.anilist_module, ai_module = EXCLUDED.ai_module, game_module = EXCLUDED.game_module",
    )
        .bind(guild_id)
        .bind(anilist_value)
        .bind(ai_value)
        .bind(game_value)
        .execute(&pool)
        .await
        .map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;
    Ok(())
}

pub async fn remove_data_activity_status_postgresql(
    server_id: String,
    anime_id: String,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query("DELETE FROM DATA.activity_data WHERE anime_id = $1 AND server_id = $2")
        .bind(anime_id)
        .bind(server_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            Error(SqlInsertError(format!(
                "Failed to insert into the table. {}",
                e
            )))
        })?;
    pool.close().await;

    Ok(())
}

pub async fn get_data_module_activation_kill_switch_status_postgresql(
) -> Result<(Option<String>, Option<bool>, Option<bool>, Option<bool>), AppError> {
    let pool = get_postgresql_pool().await?;
    let row: (Option<String>, Option<bool>, Option<bool>, Option<bool>) = sqlx::query_as(
        "SELECT id, ai_module, anilist_module, game_module FROM DATA.module_activation WHERE guild = $1",
    )
        .bind(1)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None, None));
    pool.close().await;

    Ok(row)
}

pub async fn get_one_activity_postgresql(
    server_id: String,
    anime_id: i32,
) -> Result<(Option<String>, Option<String>, Option<String>), AppError> {
    let pool = get_postgresql_pool().await?;
    let row: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id FROM DATA.activity_data WHERE anime_id = $1 AND server_id = $2",
    )
        .bind(anime_id)
        .bind(server_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None));

    pool.close().await;

    Ok(row)
}

pub async fn get_registered_user_postgresql(
    user_id: &String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let pool = get_postgresql_pool().await?;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT anilist_id, user_id FROM DATA.registered_user WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    pool.close().await;

    Ok(row)
}

pub async fn set_registered_user_postgresql(
    user_id: &String,
    username: &String,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query(
        "INSERT INTO DATA.registered_user (user_id, anilist_id) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET anilist_id = EXCLUDED.anilist_id",
    )
        .bind(user_id)
        .bind(username)
        .execute(&pool)
        .await
        .map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;

    Ok(())
}

pub async fn set_user_approximated_color_postgresql(
    user_id: &String,
    color: &String,
    pfp_url: &String,
    image: &String,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query(
        "INSERT INTO DATA.user_color (user_id, color, pfp_url, image) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET color = EXCLUDED.color, pfp_url = EXCLUDED.pfp_url, image = EXCLUDED.image",
    )
        .bind(user_id)
        .bind(color)
        .bind(pfp_url)
        .bind(image)
        .execute(&pool)
        .await
        .map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;

    Ok(())
}

pub async fn get_user_approximated_color_postgresql(
    user_id: &String,
) -> Result<UserColor, AppError> {
    let pool = get_postgresql_pool().await?;
    let row: UserColor = sqlx::query_as(
        "SELECT user_id, color, pfp_url, image FROM DATA.user_color WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(UserColor {
        user_id: None,
        color: None,
        pfp_url: None,
        image: None,
    });
    pool.close().await;

    Ok(row)
}

pub async fn get_all_server_activity_postgresql(
    server_id: &String,
) -> Result<Vec<ServerActivity>, AppError> {
    let pool = get_postgresql_pool().await?;
    let rows: Vec<ServerActivity> = sqlx::query_as(
        "SELECT
       anime_id,
       timestamp,
       server_id,
       webhook,
       episode,
       name,
       delays
       FROM DATA.activity_data WHERE server_id = $1
   ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    pool.close().await;
    Ok(rows)
}

pub async fn get_data_activity_with_server_and_anime_id_postgresql(
    anime_id: &String,
    server_id: &String,
) -> Result<Option<String>, AppError> {
    let pool = get_postgresql_pool().await?;
    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT
       webhook
       FROM DATA.activity_data WHERE server_id = $1 and anime_id = $2
   ",
    )
    .bind(server_id)
    .bind(anime_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((None, None));
    pool.close().await;
    let result = row.0;
    Ok(result)
}

pub async fn get_data_all_activity_by_server_postgresql(
    server_id: &String,
) -> Result<Vec<(String, String)>, AppError> {
    let pool = get_postgresql_pool().await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT
       anime_id, name
       FROM DATA.activity_data WHERE server_id = $1
   ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    pool.close().await;

    Ok(rows)
}

pub async fn get_all_user_approximated_color_postgres() -> Result<Vec<UserColor>, AppError> {
    let pool = get_postgresql_pool().await?;
    let row: Vec<UserColor> =
        sqlx::query_as("SELECT user_id, color, pfp_url, image FROM DATA.user_color")
            .fetch_all(&pool)
            .await
            .unwrap_or(vec![UserColor {
                user_id: None,
                color: None,
                pfp_url: None,
                image: None,
            }]);
    pool.close().await;

    Ok(row)
}
