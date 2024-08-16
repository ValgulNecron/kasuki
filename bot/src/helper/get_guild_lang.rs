use crate::config::BotConfigDetails;
use crate::get_url;
use crate::structure::database::guild_lang::{Column, Model};
use crate::structure::database::prelude::GuildLang;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
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
pub async fn get_guild_language(guild_id: String, db_config: BotConfigDetails) -> String {
    if guild_id == *"0" {
        return String::from("en");
    };
    let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
        Ok(conn) => conn,
        Err(_) => {
            return String::from("en");
        }
    };
    let guild_lang: Option<Model> = GuildLang::find()
        .filter(Column::GuildId.eq(guild_id))
        .one(&connection)
        .await
        .unwrap_or(None);

    match guild_lang {
        Some(lang) => lang.lang,
        None => String::from("en"),
    }
}
