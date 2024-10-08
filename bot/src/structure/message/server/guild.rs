use std::error::Error;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// Represents the localized guild data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains several fields which are all Strings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildLocalised {
    pub guild_id: String,
    pub guild_name: String,
    pub member: String,
    pub online: String,
    pub lang: String,
    pub premium: String,
    pub sub: String,
    pub nsfw: String,
    pub creation_date: String,
    pub owner: String,
    pub roles: String,
    pub channels: String,
    pub verification_level: String,
}

/// Loads the localized guild data.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized guild data for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized guild data.
///
/// # Returns
///
/// * `Result<GuildLocalised, AppError>` - The localized guild data,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_guild(
    guild_id: String,
    db_config: DbConfig,
) -> Result<GuildLocalised, Box<dyn Error>> {
    let path = "json/message/server/guild.json";
    load_localization(guild_id, path, db_config).await
}
