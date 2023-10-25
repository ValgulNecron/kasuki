use crate::cmd::general_module::function::sql::get_pool;

/// Asynchronously fetches the language of a given guild from the database.
///
/// This function takes the guild_id as an argument and returns the language associated with that guild.
/// The function attempts to connect to the data.db database, then executes a SQL query to select
/// the language of the given guild from the `guild_lang` table. If the given guild_id is not found
/// in the database or if there's any issue with the query, the function will still return a default
/// language of "En".
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the id of the guild.
///
/// # Examples
///
/// ```
/// let guild_id = "some id";
/// let guild_language = get_guild_langage(guild_id).await;
/// assert_eq!(guild_language, "En");
/// ```
///
/// # Returns
///
/// * `String` - The language of the guild. Returns "En" if no language is found for the guild.
///
/// # Panics
///
/// The function will panic if the database connection cannot be established or the SQL query
/// execution fails.
///
/// # Errors
///
/// This function will return a tuple (None, None) if there is any error trying to fetch the guild
/// language from the database.
pub async fn get_guild_langage(guild_id: String) -> String {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT lang, guild FROM guild_lang WHERE guild = ?")
            .bind(guild_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    let (lang, _): (Option<String>, Option<String>) = row;

    lang.unwrap_or("En".to_string())
}
