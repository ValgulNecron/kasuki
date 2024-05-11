// Importing necessary libraries and modules
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

/// ImageLocalised struct represents an image's localized data.
/// It contains a field for title.
///
/// # Struct Fields
/// `title`: A String representing the title of the image.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageLocalised {
    pub title: String,
}

/// This function loads the localization data for an image.
/// It takes a guild_id as input and returns a Result containing ImageLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<ImageLocalised, AppError>` - A Result type which is either ImageLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_image(guild_id: String) -> Result<ImageLocalised, AppError> {
    let path = "json/message/ai/image.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, ImageLocalised> = serde_json::from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse image.json. {}", e),
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
