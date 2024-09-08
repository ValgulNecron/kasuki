use std::error::Error;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// Represents the localized information data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains several fields which are all Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct InfoLocalised {
    pub title: String,
    pub desc: String,
    pub bot_name: String,
    pub bot_id: String,
    pub server_count: String,
    pub user_count: String,
    pub creation_date: String,
    pub shard: String,
    pub shard_count: String,
    pub version: String,
    pub library: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
    pub button_add_the_beta_bot: String,
    pub app_installation_count: String,
}

/// Loads the localized information data.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized information data for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized information data.
///
/// # Returns
///
/// * `Result<InfoLocalised, AppError>` - The localized information data,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.

pub async fn load_localization_info(
    guild_id: String,
    db_config: DbConfig,
) -> Result<InfoLocalised, Box<dyn Error>> {

    let path = "json/message/bot/info.json";

    load_localization(guild_id, path, db_config).await
}
