// Importing necessary libraries and modules
use std::fs;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

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
    // Read the JSON file into a String and handle any potential errors
    let json = fs::read_to_string("json/message/game/steam_game_info.json").map_err(|e| {
        AppError::new(
            format!("File steam_game_info.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

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
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the Steam game based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}