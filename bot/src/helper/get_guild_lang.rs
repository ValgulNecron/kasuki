use crate::config::BotConfigDetails;
use crate::database::data_struct::guild_language::GuildLanguage;
use crate::database::manage::dispatcher::data_dispatch::get_data_guild_language;

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
pub async fn get_guild_language(
    guild_id: String,
    db_type: String,
    db_config: BotConfigDetails,
) -> String {
    if guild_id == *"0" {
        return String::from("en");
    };

    let guild_lang: Option<GuildLanguage> = get_data_guild_language(guild_id, db_type, db_config)
        .await
        .unwrap_or(None);

    match guild_lang {
        Some(lang) => lang.lang,
        None => String::from("en"),
    }
}
