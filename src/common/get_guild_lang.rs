use crate::database::dispatcher::data_dispatch::get_data_guild_language;

/// Retrieves the language setting for a given guild.
///
/// This function takes a guild ID as a parameter and returns the language setting for that guild.
/// If the guild ID is "0", it returns "en" as the default language.
/// Otherwise, it retrieves the language setting from the database using the `get_data_guild_language` function.
/// If the language setting is not found in the database, it returns "en" as the default language.
///
/// # Arguments
///
/// * `guild_id` - A string representing the ID of the guild for which to retrieve the language setting.
///
/// # Returns
///
/// * A string representing the language setting for the given guild. If no language setting is found, it returns "en".
pub async fn get_guild_langage(guild_id: String) -> String {
    if guild_id == *"0" {
        return String::from("en");
    };

    let (lang, _): (Option<String>, Option<String>) = get_data_guild_language(guild_id)
        .await
        .unwrap_or((None, None));

    lang.unwrap_or("en".to_string())
}