// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `DeleteActivityLocalised` is a struct that represents a delete activity's localized data.
/// It contains two fields `success` and `success_desc` which are both Strings.
///
/// # Struct Fields
/// `success`: A String representing the success message of the delete activity.
/// `success_desc`: A String representing the success description of the delete activity.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteActivityLocalised {
    pub success: String,
    pub success_desc: String,
}

/// `load_localization_delete_activity` is an asynchronous function that loads the localized data for a delete activity.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `DeleteActivityLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<DeleteActivityLocalised, AppError>` - A Result type which is either DeleteActivityLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_delete_activity(
    guild_id: String,
) -> Result<DeleteActivityLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json =
        fs::read_to_string("json/message/admin/anilist/delete_activity.json").map_err(|e| {
            AppError::new(
                format!(
                    "File delete_activity.json not found or can't be read. {}",
                    e
                ),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, DeleteActivityLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse delete_activity.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_language(guild_id).await;

    // Retrieve the localized data for the delete activity based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
