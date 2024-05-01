// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `AddActivityLocalised` is a struct that represents an add activity's localized data.
/// It contains four fields `success`, `fail`, `fail_desc` and `success_desc` which are all Strings.
///
/// # Struct Fields
/// `success`: A String representing the success message of the add activity.
/// `fail`: A String representing the failure message of the add activity.
/// `fail_desc`: A String representing the failure description of the add activity.
/// `success_desc`: A String representing the success description of the add activity.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddActivityLocalised {
    pub success: String,
    pub fail: String,
    pub fail_desc: String,
    pub success_desc: String,
}

/// `load_localization_add_activity` is an asynchronous function that loads the localized data for an add activity.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `AddActivityLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<AddActivityLocalised, AppError>` - A Result type which is either AddActivityLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_add_activity(
    guild_id: String,
) -> Result<AddActivityLocalised, AppError> {
    // Open the JSON file and handle any potential errors
    let mut file = File::open("json/message/admin/anilist/add_activity.json").map_err(|e| {
        AppError::new(
            format!("File add_activity.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Initialize a new String to hold the JSON data
    let mut json = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut json).map_err(|e| {
        AppError::new(
            format!("File add_activity.json can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, AddActivityLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse add_activity.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the add activity based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
