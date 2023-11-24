use crate::cmd::anilist_module::send_activity::ActivityData;
use crate::constant::DATA_SQLITE_DB;
use crate::function::sqls::sqlite::pool::get_sqlite_pool;
use chrono::Utc;
use log::error;

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

/// Sets the language for a guild in the SQLite database.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild.
/// * `lang` - The language to set for the guild.
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

/// Retrieves activity data from SQLite database.
///
/// # Returns
///
/// A `Vec<ActivityData>` containing the retrieved data.
///
pub async fn get_data_activity_sqlite() -> Vec<ActivityData> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await;
    let now = Utc::now().timestamp().to_string();
    let rows: Vec<ActivityData> = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook, episode, name, delays FROM activity_data WHERE timestamp = ?",
    )
        .bind(now.clone())
        .fetch_all(&pool)
        .await
        .unwrap();
    rows
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
