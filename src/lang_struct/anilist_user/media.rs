// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// MediaLocalised struct represents a media's localized data.
/// It contains fields for two titles, a description, and staff text.
///
/// # Struct Fields
/// `field1_title`: A String representing the first title of the media.
/// `field2_title`: A String representing the second title of the media.
/// `desc`: A String representing the description of the media.
/// `staff_text`: A String representing the staff text of the media.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub staff_text: String,
}

/// This function loads the localization data for a media item.
/// It takes a guild_id as input and returns a Result containing MediaLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<MediaLocalised, AppError>` - A Result type which is either MediaLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_media(guild_id: String) -> Result<MediaLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist_user/media.json").map_err(|e| {
        AppError::new(
            format!("File media.json not found or can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, MediaLocalised> = serde_json::from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse media.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the media item based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}