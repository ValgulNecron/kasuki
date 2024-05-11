use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// UserLocalised struct represents a user's localized data.
/// It contains fields for manga, anime, week, day, hour, minute, weeks, days, hours, and minutes.
///
/// # Fields
/// * `manga`: A string representing the manga related data.
/// * `anime`: A string representing the anime related data.
/// * `week`: A string representing the week related data.
/// * `day`: A string representing the day related data.
/// * `hour`: A string representing the hour related data.
/// * `minute`: A string representing the minute related data.
/// * `weeks`: A string representing the weeks related data.
/// * `days`: A string representing the days related data.
/// * `hours`: A string representing the hours related data.
/// * `minutes`: A string representing the minutes related data.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLocalised {
    pub manga: String,
    pub anime: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub weeks: String,
    pub days: String,
    pub hours: String,
    pub minutes: String,
}

/// This function loads the localization data for a user.
/// It takes a guild_id as input and returns a Result containing UserLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<UserLocalised, AppError>`: A Result containing UserLocalised data or an AppError.
pub async fn load_localization_user(guild_id: String) -> Result<UserLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist_user/user.json").map_err(|e| {
        AppError::new(
            format!("File user.json not found or can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, UserLocalised> = serde_json::from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse user.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_language(guild_id).await;

    // Retrieve the localized data for the user based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
