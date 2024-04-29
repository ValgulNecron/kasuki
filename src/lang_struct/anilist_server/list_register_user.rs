use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// ListUserLocalised struct represents a user list's localized data.
/// It contains fields for title, next, and previous.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListUserLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

/// This function loads the localization data for a user list.
/// It takes a guild_id as input and returns a Result containing ListUserLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
pub async fn load_localization_list_user(guild_id: String) -> Result<ListUserLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist/list_register_user.json").map_err(|e| {
        AppError::new(
            format!(
                "File list_register_user.json not found or can't be read. {}",
                e
            ),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, ListUserLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse list_register_user.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the user list based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
