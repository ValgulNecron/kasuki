use tracing::{error, trace};

use crate::constant::SQLITE_DB_PATH;
use crate::database::data_struct::guild_language::GuildLanguage;
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::data_struct::ping_history::PingHistory;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::data_struct::server_activity::{
    ServerActivity, ServerActivityFull, SmallServerActivity,
};
use crate::database::data_struct::user_color::UserColor;
use crate::database::manage::sqlite::pool::get_sqlite_pool;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Inserts or replaces a record in the `ping_history` table of a SQLite database.
///
/// The function takes a 'shard_id' and a 'latency', both of type `String`, as input.
/// It attempts to insert or replace a record with these values into the `ping_history` table.
/// The `shard_id` and `latency` are most likely related to a latency reported for a specific shard ID.
/// The current timestamp is also stored with each record.
/// The function is asynchronous and returns nothing.
///
/// # Arguments
///
/// * `shard_id` - A String containing the ID of a shard.
/// * `latency` - A String containing the latency value.
///
/// # Errors
///
/// This function will log errors encountered when executing the SQL command, but does not return them.
pub async fn set_data_ping_history_sqlite(ping_history: PingHistory) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO ping_history (shard_id, timestamp, ping) VALUES (?, ?, ?)",
    )
    .bind(ping_history.shard_id)
    .bind(ping_history.timestamp)
    .bind(ping_history.ping)
    .execute(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to insert into the table. {}", e),
            ErrorType::Database,
            ErrorResponseType::Unknown,
        )
    })?;
    pool.close().await;
    Ok(())
}

/// This function retrieves language data for a guild from a SQLite database.
///
/// # Arguments
///
/// * `guild_id` - A string representing the ID of the guild.
///
/// # Returns
///
/// A tuple containing the language and guild ID as optional strings.
/// If the data is found in the database, it will be returned.
/// If not found, both values will be `None`.
pub async fn get_data_guild_language_sqlite(
    guild_id: String,
) -> Result<Option<GuildLanguage>, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: Option<GuildLanguage> =
        sqlx::query_as("SELECT lang, guild FROM guild_lang WHERE guild = ?")
            .bind(guild_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);
    pool.close().await;
    Ok(row)
}

/// Sets the language for a guild in the SQLite database.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild.
/// * `lang_struct` - The language to set for the guild.
pub async fn set_data_guild_language_sqlite(guild_language: GuildLanguage) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query("INSERT OR REPLACE INTO guild_lang (guild, lang) VALUES (?, ?)")
        .bind(guild_language.guild)
        .bind(guild_language.lang)
        .execute(&pool)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to insert into the table. {}", e),
                ErrorType::Database,
                ErrorResponseType::Unknown,
            )
        })?;
    pool.close().await;
    Ok(())
}

/// Retrieves activity data from SQLite database.
///
/// # Returns
///
/// A `Vec<ActivityData>` containing the retrieved data.
///
pub async fn get_data_activity_sqlite(now: String) -> Result<Vec<ServerActivityFull>, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let rows: Vec<ServerActivityFull> = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook, episode, name, delays, image FROM activity_data WHERE timestamp = ?",
    )
        .bind(now)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Ok(rows)
}

/// Sets data activity in SQLite database.
///
/// # Arguments
///
/// * `anime_id` - The ID of the anime.
/// * `timestamp` - The timestamp.
/// * `guild_id` - The ID of the guild.
/// * `webhook` - The webhook URL.
/// * `episode` - The episode number.
/// * `name` - The name of the anime.
/// * `delays` - The delays.
///
pub async fn set_data_activity_sqlite(
    server_activity_full: ServerActivityFull,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook, episode, name, delays, image) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
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
        .await.map_err(|e|
    AppError::new(
        format!("Failed to insert into the table. {}", e),
        ErrorType::Database,
        ErrorResponseType::Unknown,
    ))?;
    pool.close().await;
    Ok(())
}

/// Retrieves the activation status of various modules for a given guild from the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch the activation status of various modules for the given guild.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `guild_id` - A reference to a String representing the ID of the guild.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing an `ActivationStatusModule` struct if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_data_module_activation_status_sqlite(
    guild_id: &String,
) -> Result<ActivationStatusModule, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: ActivationStatusModule = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module, game_module, new_member, anime, vn FROM module_activation WHERE guild = ?",
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

/// Sets the activation status of various modules for a given guild in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to set the activation status of various modules for the given guild.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `guild_id` - A reference to a String representing the ID of the guild.
/// * `anilist_value` - A boolean representing the activation status of the Anilist module.
/// * `ai_value` - A boolean representing the activation status of the AI module.
/// * `game_value` - A boolean representing the activation status of the Game module.
/// * `new_member_value` - A boolean representing the activation status of the New Member module.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn set_data_module_activation_status_sqlite(
    activation_status_module: ActivationStatusModule,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO module_activation (guild_id, anilist_module, ai_module, game_module, anime, new_member, vn) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(activation_status_module.id)
        .bind(activation_status_module.anilist_module)
        .bind(activation_status_module.ai_module)
        .bind(activation_status_module.game_module)
        .bind(activation_status_module.anime)
        .bind(activation_status_module.new_member)
        .bind(activation_status_module.vn)
        .execute(&pool)
        .await
        .map_err(|e|
        AppError::new(
            format!("Failed to insert into the table. {}", e),
            ErrorType::Database,
            ErrorResponseType::Unknown,
        ))?;
    pool.close().await;
    Ok(())
}

/// Removes a record from the `activity_data` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to delete a record from the `activity_data` table where the `anime_id` and `server_id` match the provided values.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `server_id` - A String representing the ID of the server.
/// * `anime_id` - A String representing the ID of the anime.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn remove_data_activity_status_sqlite(
    server_id: String,
    anime_id: String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query("DELETE FROM activity_data WHERE anime_id = ? AND server_id = ?")
        .bind(anime_id)
        .bind(server_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to delete from the table. {}", e),
                ErrorType::Database,
                ErrorResponseType::Unknown,
            )
        })?;
    pool.close().await;

    Ok(())
}

/// Retrieves the activation status of various modules for a specific guild from the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch the activation status of various modules for the guild with ID 1.
/// 3. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing an `ActivationStatusModule` struct if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_data_module_activation_kill_switch_status_sqlite(
) -> Result<ActivationStatusModule, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: ActivationStatusModule = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module, game_module, new_member, anime, vn FROM module_activation WHERE guild = 1",
    )
        .fetch_one(&pool)
        .await
        .unwrap_or(
            ActivationStatusModule {
                guild_id: None,
                ai_module: None,
                anilist_module: None,
                game_module: None,
                new_member: None,
                anime: None,
                vn: None,
            },
        );
    pool.close().await;

    Ok(row)
}

/// Retrieves a specific activity from the `activity_data` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch a specific activity from the `activity_data` table where the `anime_id` and `server_id` match the provided values.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `server_id` - A String representing the ID of the server.
/// * `anime_id` - An integer representing the ID of the anime.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a tuple of optional Strings if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_one_activity_sqlite(
    server_id: String,
    anime_id: i32,
) -> Result<SmallServerActivity, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: SmallServerActivity = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id FROM activity_data WHERE anime_id = ? AND server_id = ?",
    )
        .bind(anime_id)
        .bind(server_id)
        .fetch_one(&pool)
        .await.unwrap_or_else(|e| {
        error!(?e);
        SmallServerActivity {
            anime_id: None,
            timestamp: None,
            server_id: None,
        }
    });
    trace!(?row);
    pool.close().await;

    Ok(row)
}

/// Retrieves a registered user from the `registered_user` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch a registered user from the `registered_user` table where the `user_id` matches the provided value.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `user_id` - A reference to a String representing the ID of the user.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a tuple of optional Strings if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_registered_user_sqlite(
    user_id: &String,
) -> Result<Option<RegisteredUser>, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: Option<RegisteredUser> =
        sqlx::query_as("SELECT anilist_id, user_id FROM registered_user WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);
    pool.close().await;

    Ok(row)
}

/// Inserts or replaces a record in the `registered_user` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to insert or replace a record in the `registered_user` table where the `user_id` and `anilist_id` match the provided values.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `user_id` - A reference to a String representing the ID of the user.
/// * `username` - A reference to a String representing the username of the user.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn set_registered_user_sqlite(registered_user: RegisteredUser) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ =
        sqlx::query("INSERT OR REPLACE INTO registered_user (user_id, anilist_id) VALUES (?, ?)")
            .bind(registered_user.user_id)
            .bind(registered_user.anilist_id)
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to insert into the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::Unknown,
                )
            })?;
    pool.close().await;

    Ok(())
}

/// Inserts or replaces a record in the `user_color` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to insert or replace a record in the `user_color` table where the `user_id`, `color`, `pfp_url`, and `image` match the provided values.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `user_id` - A reference to a String representing the ID of the user.
/// * `color` - A reference to a String representing the color associated with the user.
/// * `pfp_url` - A reference to a String representing the URL of the user's profile picture.
/// * `image` - A reference to a String representing the image associated with the user.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn set_user_approximated_color_sqlite(
    user_id: &String,
    color: &String,
    pfp_url: &String,
    image: &String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO user_color (user_id, color, pfp_url, image) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(color)
    .bind(pfp_url)
    .bind(image)
    .execute(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to insert into the table. {}", e),
            ErrorType::Database,
            ErrorResponseType::Unknown,
        )
    })?;
    pool.close().await;

    Ok(())
}

/// Retrieves the approximated color for a user from the `user_color` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch a user's approximated color from the `user_color` table where the `user_id` matches the provided value.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `user_id` - A reference to a String representing the ID of the user.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a `UserColor` struct if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_user_approximated_color_sqlite(user_id: &String) -> Result<UserColor, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: UserColor =
        sqlx::query_as("SELECT user_id, color, pfp_url, image FROM user_color WHERE user_id = ?")
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

/// Retrieves all server activity for a specific server from the `activity_data` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch all server activity from the `activity_data` table where the `server_id` matches the provided value.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `server_id` - A reference to a String representing the ID of the server.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a vector of `ServerActivity` structs if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_all_server_activity_sqlite(
    server_id: &String,
) -> Result<Vec<ServerActivity>, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let rows: Vec<ServerActivity> = sqlx::query_as(
        "SELECT
       anime_id,
       timestamp,
       server_id,
       webhook,
       episode,
       name,
       delays
       FROM activity_data WHERE server_id = ?
   ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(rows)
}

/// Retrieves all user approximated colors from the `user_color` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch all user approximated colors from the `user_color` table.
/// 3. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a vector of `UserColor` structs if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_all_user_approximated_color_sqlite() -> Result<Vec<UserColor>, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
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

/// Retrieves all activity data for a specific server from the `activity_data` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch all activity data from the `activity_data` table where the `server_id` matches the provided value.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `server_id` - A reference to a String representing the ID of the server.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a vector of tuples (each containing a String representing the anime ID and a String representing the name) if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_data_all_activity_by_server_sqlite(
    server_id: &String,
) -> Result<Vec<(String, String)>, AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let rows = sqlx::query_as(
        "SELECT
           anime_id, name
           FROM activity_data WHERE server_id = ?
       ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    pool.close().await;

    Ok(rows)
}

/// Inserts or replaces a record in the `server_image` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to insert or replace a record in the `server_image` table where the `server_id`, `type`, `image`, and `image_url` match the provided values.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `server_id` - A reference to a String representing the ID of the server.
/// * `image_type` - A reference to a String representing the type of the image.
/// * `image` - A reference to a String representing the image.
/// * `image_url` - A reference to a String representing the URL of the image.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn set_server_image_sqlite(
    server_id: &String,
    image_type: &String,
    image: &String,
    image_url: &String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO server_image (server_id, type, image, image_url) VALUES (?, ?, ?, ?)",
    )
        .bind(server_id)
        .bind(image_type)
        .bind(image)
        .bind(image_url)
        .execute(&pool)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to insert into the table. {}", e),
                ErrorType::Database,
                ErrorResponseType::Unknown,
            )
        })?;
    pool.close().await;

    Ok(())
}

/// Retrieves an image for a specific server from the `server_image` table in the SQLite database.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Executes a SQL query to fetch an image from the `server_image` table where the `server_id` and `type` match the provided values.
/// 3. Closes the connection pool.
///
/// # Arguments
///
/// * `server_id` - A reference to a String representing the ID of the server.
/// * `image_type` - A reference to a String representing the type of the image.
///
/// # Returns
///
/// * A Result that is either an Ok variant containing a tuple of optional Strings (representing the image URL and the image) if the operation was successful, or an Err variant with an `AppError` if the operation failed.
pub async fn get_server_image_sqlite(
    server_id: &String,
    image_type: &String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;
    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT image_url, image FROM server_image WHERE server_id = ? and type = ?",
    )
    .bind(server_id)
    .bind(image_type)
    .fetch_one(&pool)
    .await
    .unwrap_or((None, None));
    pool.close().await;
    Ok(row)
}
