use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

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
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
    pub button_add_the_beta_bot: String,
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
pub async fn load_localization_info(guild_id: String) -> Result<InfoLocalised, AppError> {
    let path = "json/message/bot/info.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, InfoLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse info.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_language(guild_id).await;

    // Return the localized data for the language or an error if the language is not found.
    Ok(json_data
        .get(lang_choice.as_str())
        .cloned()
        .unwrap_or(json_data.get("en").unwrap().clone()))
}
