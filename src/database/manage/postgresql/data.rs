use std::error::Error;

use crate::config::BotConfigDetails;
use crate::database::data_struct::guild_language::GuildLanguage;
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::data_struct::ping_history::PingHistory;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::data_struct::server_activity::{
    ServerActivity, ServerActivityFull, SmallServerActivity,
};
use crate::database::data_struct::user_color::UserColor;
use crate::database::manage::postgresql::pool::get_postgresql_pool;
use crate::helper::error_management::error_enum::UnknownResponseError;

/// Inserts or updates a ping history record in the PostgreSQL database.
///
/// This function takes two parameters: `shard_id` and `latency`.
/// It inserts these values into the `DATA.ping_history` table. If a record with the same `shard_id` already exists, it updates the existing record with the new values.
///
/// # Parameters
///
/// * `shard_id`: A `String` that represents the shard id of the ping history record.
/// * `latency`: A `String` that represents the latency of the ping history record.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_data_ping_history_postgresql(
    ping_history: PingHistory,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO ping_history (shard_id, timestamp, ping) VALUES ($1, $2, $3) ON CONFLICT (shard_id) DO UPDATE SET timestamp = EXCLUDED.timestamp, ping = EXCLUDED.ping",
    )
        .bind(ping_history.shard_id)
        .bind(ping_history.timestamp)
        .bind(ping_history.ping)
        .execute(&pool)
        .await
        .map_err(|e|
        UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;
    Ok(())
}

/// Retrieves a guild language record from the PostgreSQL database.
///
/// This function takes a `guild_id` parameter which is used to query the database.
/// It fetches the `lang` and `guild` fields from the `DATA.guild_lang` table where the `guild` matches the input `guild_id`.
///
/// # Parameters
///
/// * `guild_id`: A `String` that represents the guild id of the guild language record.
///
/// # Returns
///
/// * A Result that is either a tuple containing Option<String>, Option<String> if the operation was successful and the guild language record exists, or (None, None) if the guild language record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_data_guild_language_postgresql(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<Option<GuildLanguage>, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: Option<GuildLanguage> =
        sqlx::query_as("SELECT lang, guild FROM guild_lang WHERE guild = $1")
            .bind(guild_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);
    pool.close().await;
    Ok(row)
}

/// Sets a guild language record in the PostgreSQL database.
///
/// This function takes two parameters: `guild_id` and `lang`.
/// It inserts these values into the `DATA.guild_lang` table. If a record with the same `guild_id` already exists, it updates the existing record with the new `lang`.
///
/// # Parameters
///
/// * `guild_id`: A `String` reference that represents the guild id of the guild language record.
/// * `lang`: A `String` reference that represents the language of the guild language record.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_data_guild_language_postgresql(
    guild_language: GuildLanguage,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO guild_lang (guild, lang) VALUES ($1, $2) ON CONFLICT (guild) DO UPDATE SET lang = EXCLUDED.lang",
    )
        .bind(guild_language.guild)
        .bind(guild_language.lang)
        .execute(&pool)
        .await
        .map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;
    Ok(())
}

/// Retrieves activity data records from the PostgreSQL database.
///
/// This function takes a `now` parameter which is used to query the database.
/// It fetches all fields from the `DATA.activity_data` table where the `timestamp` matches the input `now`.
///
/// # Parameters
///
/// * `now`: A `String` that represents the timestamp of the activity data records.
///
/// # Returns
///
/// * A Result that is either a Vec<ActivityData> if the operation was successful and the activity data records exist, or an empty Vec<ActivityData> if the activity data records do not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_data_activity_postgresql(
    now: String,
    db_config: BotConfigDetails,
) -> Result<Vec<ServerActivityFull>, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let rows: Vec<ServerActivityFull> = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook, episode, name, delays, image FROM activity_data WHERE timestamp = $1",
    )
        .bind(now)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Ok(rows)
}

/// Inserts or updates an activity data record in the PostgreSQL database.
///
/// This function takes a `server_activity_full` parameter of type `ServerActivityFull`.
/// It inserts these values into the `DATA.activity_data` table. If a record with the same `anime_id` already exists, it updates the existing record with the new values.
///
/// # Parameters
///
/// * `server_activity_full`: A `ServerActivityFull` that represents the activity data record.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_data_activity_postgresql(
    server_activity_full: ServerActivityFull,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO activity_data (anime_id, timestamp, server_id, webhook, episode, name, delays, image) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)\
         ON CONFLICT (anime_id) DO UPDATE SET timestamp = EXCLUDED.timestamp, server_id = EXCLUDED.server_id, webhook = EXCLUDED.webhook, episode = EXCLUDED.episode, name = EXCLUDED.name, delays = EXCLUDED.delays",
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
        .await.map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;
    Ok(())
}

/// Retrieves a module activation status record from the PostgreSQL database.
///
/// This function takes a `guild_id` parameter which is used to query the database.
/// It fetches all fields from the `DATA.module_activation` table where the `guild` matches the input `guild_id`.
///
/// # Parameters
///
/// * `guild_id`: A `String` reference that represents the guild id of the module activation status record.
///
/// # Returns
///
/// * A Result that is either an ActivationStatusModule if the operation was successful and the module activation status record exists, or an ActivationStatusModule with None fields if the module activation status record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_data_module_activation_status_postgresql(
    guild_id: &String,
    db_config: BotConfigDetails,
) -> Result<ActivationStatusModule, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: ActivationStatusModule = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module, game_module, new_member, anime, vn FROM module_activation WHERE guild = $1",
    )
        .bind(guild_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(ActivationStatusModule {
            guild_id: None,
            ai_module: None,
            anilist_module: None,
            game_module: None,
            new_member: None,
            anime: None,
            vn: None,
        });
    pool.close().await;
    Ok(row)
}

/// Sets a module activation status record in the PostgreSQL database.
///
/// This function takes four parameters: `guild_id`, `anilist_value`, `ai_value`, `game_value`, and `new_member_value`.
/// It inserts these values into the `DATA.module_activation` table. If a record with the same `guild_id` already exists, it updates the existing record with the new values.
///
/// # Parameters
///
/// * `guild_id`: A `String` reference that represents the guild id of the module activation status record.
/// * `anilist_value`: A `bool` that represents the activation status of the Anilist module.
/// * `ai_value`: A `bool` that represents the activation status of the AI module.
/// * `game_value`: A `bool` that represents the activation status of the Game module.
/// * `new_member_value`: A `bool` that represents the activation status of the New Member module.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_data_module_activation_status_postgresql(
    activation_status_module: ActivationStatusModule,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO module_activation (guild_id, anilist_module, ai_module, game_module, new_member, vn) VALUES ($1, $2, $3, $4, $5, $6) \
        ON CONFLICT (guild_id) DO UPDATE SET anilist_module = EXCLUDED.anilist_module, ai_module = EXCLUDED.ai_module, game_module = EXCLUDED.game_module, new_member = EXCLUDED.new_member, vn = EXCLUDED.vn",
    )
        .bind(activation_status_module.guild_id)
        .bind(activation_status_module.anilist_module)
        .bind(activation_status_module.ai_module)
        .bind(activation_status_module.game_module)
        .bind(activation_status_module.new_member)
        .bind(activation_status_module.vn)
        .execute(&pool)
        .await
        .map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;
    Ok(())
}
pub async fn set_data_kill_switch_activation_status_postgresql(
    activation_status_module: ActivationStatusModule,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO global_kill_switch (guild_id, anilist_module, ai_module, game_module, new_member, vn) VALUES ($1, $2, $3, $4, $5, $6) \
        ON CONFLICT (guild_id) DO UPDATE SET anilist_module = EXCLUDED.anilist_module, ai_module = EXCLUDED.ai_module, game_module = EXCLUDED.game_module, new_member = EXCLUDED.new_member, vn = EXCLUDED.vn",
    )
        .bind(1)
        .bind(activation_status_module.anilist_module)
        .bind(activation_status_module.ai_module)
        .bind(activation_status_module.game_module)
        .bind(activation_status_module.new_member)
        .bind(activation_status_module.vn)
        .execute(&pool)
        .await
        .map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;
    Ok(())
}
/// Removes an activity data record from the PostgreSQL database.
///
/// This function takes two parameters: `server_id` and `anime_id`.
/// It deletes the record from the `DATA.activity_data` table where the `anime_id` and `server_id` match the input parameters.
///
/// # Parameters
///
/// * `server_id`: A `String` that represents the server id of the activity data record.
/// * `anime_id`: A `String` that represents the anime id of the activity data record.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn remove_data_activity_status_postgresql(
    server_id: String,
    anime_id: String,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query("DELETE FROM activity_data WHERE anime_id = $1 AND server_id = $2")
        .bind(anime_id)
        .bind(server_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e))
        })?;
    pool.close().await;

    Ok(())
}

/// Retrieves the module activation status for the kill switch from the PostgreSQL database.
///
/// This function queries the `DATA.module_activation` table where the `guild` is 1 (representing the kill switch).
///
/// # Returns
///
/// * A Result that is either an ActivationStatusModule if the operation was successful and the module activation status record exists, or an ActivationStatusModule with None fields if the module activation status record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_data_module_activation_kill_switch_status_postgresql(
    db_config: BotConfigDetails,
) -> Result<ActivationStatusModule, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: ActivationStatusModule = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module, game_module, new_member, anime, vn FROM global_kill_switch WHERE guild = $1",
    )
        .bind(1)
        .fetch_one(&pool)
        .await
        .unwrap_or(ActivationStatusModule {
            guild_id: None,
            ai_module: None,
            anilist_module: None,
            game_module: None,
            new_member: None,
            anime: None,
            vn: None,
        });
    pool.close().await;

    Ok(row)
}

/// Retrieves a specific activity data record from the PostgreSQL database.
///
/// This function takes two parameters: `server_id` and `anime_id`.
/// It fetches the `anime_id`, `timestamp`, and `server_id` fields from the `DATA.activity_data` table where the `anime_id` and `server_id` match the input parameters.
///
/// # Parameters
///
/// * `server_id`: A `String` that represents the server id of the activity data record.
/// * `anime_id`: An `i32` that represents the anime id of the activity data record.
///
/// # Returns
///
/// * A Result that is either a tuple containing Option<String>, Option<String>, Option<String> if the operation was successful and the activity data record exists, or (None, None, None) if the activity data record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_one_activity_postgresql(
    server_id: String,
    anime_id: i32,
    db_config: BotConfigDetails,
) -> Result<SmallServerActivity, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: SmallServerActivity = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id FROM activity_data WHERE anime_id = $1 AND server_id = $2",
    )
        .bind(anime_id)
        .bind(server_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(SmallServerActivity {
            anime_id: None,
            timestamp: None,
            server_id: None,
        });

    pool.close().await;

    Ok(row)
}

/// Retrieves a registered user record from the PostgreSQL database.
///
/// This function takes a `user_id` parameter which is used to query the database.
/// It fetches the `anilist_id` and `user_id` fields from the `DATA.registered_user` table where the `user_id` matches the input `user_id`.
///
/// # Parameters
///
/// * `user_id`: A `String` reference that represents the user id of the registered user record.
///
/// # Returns
///
/// * A Result that is either a tuple containing Option<String>, Option<String> if the operation was successful and the registered user record exists, or (None, None) if the registered user record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_registered_user_postgresql(
    user_id: &String,
    db_config: BotConfigDetails,
) -> Result<Option<RegisteredUser>, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: Option<RegisteredUser> =
        sqlx::query_as("SELECT anilist_id, user_id FROM registered_user WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);
    pool.close().await;

    Ok(row)
}

/// Sets a registered user record in the PostgreSQL database.
///
/// This function takes two parameters: `user_id` and `username`.
/// It inserts these values into the `DATA.registered_user` table. If a record with the same `user_id` already exists, it updates the existing record with the new `username`.
///
/// # Parameters
///
/// * `user_id`: A `String` reference that represents the user id of the registered user record.
/// * `username`: A `String` reference that represents the username of the registered user record.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_registered_user_postgresql(
    registered_user: RegisteredUser,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO registered_user (user_id, anilist_id) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET anilist_id = EXCLUDED.anilist_id",
    )
        .bind(registered_user.user_id)
        .bind(registered_user.anilist_id)
        .execute(&pool)
        .await
        .map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;

    Ok(())
}

/// Sets a user's approximated color record in the PostgreSQL database.
///
/// This function takes four parameters: `user_id`, `color`, `pfp_url`, and `image`.
/// It inserts these values into the `DATA.user_color` table. If a record with the same `user_id` already exists, it updates the existing record with the new values.
///
/// # Parameters
///
/// * `user_id`: A `String` reference that represents the user id of the user color record.
/// * `color`: A `String` reference that represents the approximated color of the user.
/// * `pfp_url`: A `String` reference that represents the profile picture URL of the user.
/// * `image`: A `String` reference that represents the image of the user.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_user_approximated_color_postgresql(
    user_id: &String,
    color: &String,
    pfp_url: &String,
    image: &String,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO user_color (user_id, color, pfp_url, image) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET color = EXCLUDED.color, pfp_url = EXCLUDED.pfp_url, image = EXCLUDED.image",
    )
        .bind(user_id)
        .bind(color)
        .bind(pfp_url)
        .bind(image)
        .execute(&pool)
        .await
        .map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;

    Ok(())
}

/// Retrieves a user's approximated color record from the PostgreSQL database.
///
/// This function takes a `user_id` parameter which is used to query the database.
/// It fetches the `user_id`, `color`, `pfp_url`, and `image` fields from the `DATA.user_color` table where the `user_id` matches the input `user_id`.
///
/// # Parameters
///
/// * `user_id`: A `String` reference that represents the user id of the user color record.
///
/// # Returns
///
/// * A Result that is either a UserColor if the operation was successful and the user color record exists, or a UserColor with None fields if the user color record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_user_approximated_color_postgresql(
    user_id: &String,
    db_config: BotConfigDetails,
) -> Result<UserColor, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: UserColor = sqlx::query_as(
        "SELECT user_id, color, pfp_url, image FROM user_color WHERE user_id = $1",
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

/// Retrieves all server activity records associated with a specific server from the PostgreSQL database.
///
/// This function takes a `server_id` parameter which is used to query the database.
/// It fetches all fields from the `DATA.activity_data` table where the `server_id` matches the input `server_id`.
///
/// # Parameters
///
/// * `server_id`: A `String` reference that represents the server id of the activity data records.
///
/// # Returns
///
/// * A Result that is either a Vec<ServerActivity> if the operation was successful and the activity data records exist, or an empty Vec<ServerActivity> if the activity data records do not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_all_server_activity_postgresql(
    server_id: &String,
    db_config: BotConfigDetails,
) -> Result<Vec<ServerActivity>, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let rows: Vec<ServerActivity> = sqlx::query_as(
        "SELECT
       anime_id,
       timestamp,
       server_id,
       webhook,
       episode,
       name,
       delays
       FROM activity_data WHERE server_id = $1
   ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    pool.close().await;
    Ok(rows)
}

/// Retrieves all activity data records associated with a specific server from the PostgreSQL database.
///
/// This function takes a `server_id` parameter which is used to query the database.
/// It fetches the `anime_id` and `name` fields from the `DATA.activity_data` table where the `server_id` matches the input `server_id`.
///
/// # Parameters
///
/// * `server_id`: A `String` reference that represents the server id of the activity data records.
///
/// # Returns
///
/// * A Result that is either a Vec<(String, String)> if the operation was successful and the activity data records exist, or an empty Vec<(String, String)> if the activity data records do not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_data_all_activity_by_server_postgresql(
    server_id: &String,
    db_config: BotConfigDetails,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT
       anime_id, name
       FROM activity_data WHERE server_id = $1
   ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    pool.close().await;

    Ok(rows)
}

/// Retrieves all user's approximated color records from the PostgreSQL database.
///
/// This function queries the `DATA.user_color` table and fetches all records.
///
/// # Returns
///
/// * A Result that is either a Vec<UserColor> if the operation was successful and the user color records exist, or a Vec<UserColor> with None fields if the user color records do not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_all_user_approximated_color_postgres(
    db_config: BotConfigDetails,
) -> Result<Vec<UserColor>, Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: Vec<UserColor> =
        sqlx::query_as("SELECT user_id, color, pfp_url, image FROM user_color")
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

/// Sets a server image record in the PostgreSQL database.
///
/// This function takes four parameters: `server_id`, `image_type`, `image`, and `image_url`.
/// It inserts these values into the `DATA.server_image` table. If a record with the same `server_id` and `image_type` already exists, it updates the existing record with the new `image`.
///
/// # Parameters
///
/// * `server_id`: A `String` reference that represents the server id of the server image record.
/// * `image_type`: A `String` reference that represents the type of the server image record.
/// * `image`: A `String` reference that represents the image of the server image record.
/// * `image_url`: A `String` reference that represents the image URL of the server image record.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_server_image_postgresql(
    server_id: &String,
    image_type: &String,
    image: &String,
    image_url: &String,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    sqlx::query(
        "INSERT INTO server_image (server_id, image_type, image, image_url) VALUES ($1, $2, $3, $4) ON CONFLICT (server_id, image_type) DO UPDATE SET image = EXCLUDED.image",
    )
        .bind(server_id)
        .bind(image_type)
        .bind(image)
        .bind(image_url)
        .execute(&pool)
        .await
        .map_err(|e| UnknownResponseError::Database(format!("Failed to insert into the table. {:#?}", e)))?;
    pool.close().await;

    Ok(())
}

/// Retrieves a server image record from the PostgreSQL database.
///
/// This function takes two parameters: `server_id` and `image_type`.
/// It fetches the `image_url` and `image` fields from the `DATA.server_image` table where the `server_id` and `image_type` match the input parameters.
///
/// # Parameters
///
/// * `server_id`: A `String` reference that represents the server id of the server image record.
/// * `image_type`: A `String` reference that represents the type of the server image record.
///
/// # Returns
///
/// * A Result that is either a tuple containing Option<String>, Option<String> if the operation was successful and the server image record exists, or (None, None) if the server image record does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_server_image_postgresql(
    server_id: &String,
    image_type: &String,
    db_config: BotConfigDetails,
) -> Result<(Option<String>, Option<String>), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;
    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT image_url, image FROM server_image WHERE server_id = $1 and image_type = $2",
    )
    .bind(server_id)
    .bind(image_type)
    .fetch_one(&pool)
    .await
    .unwrap_or((None, None));
    pool.close().await;
    Ok(row)
}
