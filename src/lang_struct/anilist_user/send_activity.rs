// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// SendActivityLocalised struct represents a send activity's localized data.
/// It contains fields for title and description.
///
/// # Fields
/// * `title`: A string representing the title related data.
/// * `desc`: A string representing the description related data.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SendActivityLocalised {
    pub title: String,
    pub desc: String,
}

/// This function loads the localization data for a send activity.
/// It takes a guild_id as input and returns a Result containing SendActivityLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<SendActivityLocalised, AppError>`: A Result containing SendActivityLocalised data or an AppError.
pub async fn load_localization_send_activity(
    guild_id: String,
) -> Result<SendActivityLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist_user/send_activity.json").map_err(|e| {
        AppError::new(
            format!("File send_activity.json not found or can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, SendActivityLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse send_activity.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the send activity based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}