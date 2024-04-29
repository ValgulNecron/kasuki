use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `PFPServerLocalisedImage` is a struct that represents a server's localized data for profile picture image.
/// It contains a single field `title` which is a String.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PFPServerLocalisedImage {
    pub title: String,
}

/// `load_localization_pfp_server_image` is an asynchronous function that loads the localized data for a server's profile picture image.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `PFPServerLocalisedImage` struct or an `AppError`.
///
/// # Errors
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_pfp_server_image(
    guild_id: String,
) -> Result<PFPServerLocalisedImage, AppError> {
    // Read the JSON file into a String.
    let json =
        fs::read_to_string("json/message/general/generate_image_pfp_server.json").map_err(|e| {
            AppError::new(
                format!("File generate_image_pfp_server.json not found. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, PFPServerLocalisedImage> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse generate_image_pfp_server.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_langage(guild_id).await;

    // Return the localized data for the server's profile picture image or an error if the language is not found.
    json_data.get(lang_choice.as_str()).cloned().ok_or_else(|| {
        AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        )
    })
}
