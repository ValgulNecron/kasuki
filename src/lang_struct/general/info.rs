use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `InfoLocalised` is a struct that represents a information's localized data.
/// It contains several fields which are all Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InfoLocalised {
    pub title: String,
    pub desc: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
    pub button_add_the_beta_bot: String,
}

/// `load_localization_info` is an asynchronous function that loads the localized data for information.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `InfoLocalised` struct or an `AppError`.
///
/// # Errors
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_info(guild_id: String) -> Result<InfoLocalised, AppError> {
    // Read the JSON file into a String.
    let json = fs::read_to_string("json/message/general/info.json").map_err(|e| {
        AppError::new(
            format!("File info.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, InfoLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse info.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_langage(guild_id).await;

    // Return the localized data for the information or an error if the language is not found.
    json_data.get(lang_choice.as_str()).cloned().ok_or_else(|| {
        AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        )
    })
}
