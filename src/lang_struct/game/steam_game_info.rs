use std::fs;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Struct representing the localized information of a Steam steam.
/// Each field represents a different piece of information about the steam.
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

/// Function to load the localized information of a Steam steam.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild ID.
///
/// # Returns
///
/// * `Result<SteamGameInfoLocalised, AppError>` - On success, the function returns `Ok` wrapping the localized steam information. On failure, it returns `Err` wrapping an `AppError`.
pub async fn load_localization_steam_game_info(
    guild_id: String,
) -> Result<SteamGameInfoLocalised, AppError> {
    // Reading the JSON file containing the steam information
    let json = fs::read_to_string("json/message/game/steam_game_info.json").map_err(|e| {
        AppError::new(
            format!("File steam_game_info.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parsing the JSON data into a HashMap
    let json_data: std::collections::HashMap<String, SteamGameInfoLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse steam_game_info.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Getting the language choice based on the guild ID
    let lang_choice = get_guild_langage(guild_id).await;

    // Returning the localized steam information based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
