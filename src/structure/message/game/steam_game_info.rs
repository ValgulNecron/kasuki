// Importing necessary libraries and modules
use std::fs;

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

/// `SteamGameInfoLocalised` is a struct that represents a Steam game's localized data.
/// It contains several fields which are all Strings.
///
/// # Struct Fields
/// `field1` to `field7`, `free`, `coming_soon`, `tba`: Strings representing different pieces of information about the Steam game.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SteamGameInfoLocalised {
    pub field1: String,
    pub field2: String,
    pub field3: String,
    pub field4: String,
    pub field5: String,
    pub field6: String,
    pub field7: String,
    pub free: String,
    pub coming_soon: String,
    pub tba: String,
}

/// `load_localization_steam_game_info` is an asynchronous function that loads the localized data for a Steam game.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `SteamGameInfoLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<SteamGameInfoLocalised, AppError>` - A Result type which is either SteamGameInfoLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_steam_game_info(
    guild_id: String,
) -> Result<SteamGameInfoLocalised, AppError> {
    let path = "json/message/game/steam_game_info.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: std::collections::HashMap<String, SteamGameInfoLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse steam_game_info.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_language(guild_id).await;

    // Return the localized data for the language or an error if the language is not found.
    Ok(json_data
        .get(lang_choice.as_str())
        .cloned()
        .unwrap_or(json_data.get("en").unwrap().clone()))
}
