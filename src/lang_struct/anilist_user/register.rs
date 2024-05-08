use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// RegisterLocalised struct represents a register's localized data.
/// It contains a field for description.
///
/// # Struct Fields
/// `desc`: A String representing the description of the register.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalised {
    pub desc: String,
}

/// This function loads the localization data for a register.
/// It takes a guild_id as input and returns a Result containing RegisterLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<RegisterLocalised, AppError>` - A Result type which is either RegisterLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_register(guild_id: String) -> Result<RegisterLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist_user/register.json").map_err(|e| {
        AppError::new(
            format!("File register.json not found or can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, RegisterLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse register.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the register based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
