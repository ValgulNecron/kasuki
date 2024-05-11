use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

/// StaffLocalised struct represents a staff's localized data.
/// It contains fields for two titles, description, date of birth, and date of death.
///
/// # Fields
/// * `field1_title`: A string representing the first title related data.
/// * `field2_title`: A string representing the second title related data.
/// * `desc`: A string representing the description related data.
/// * `date_of_birth`: A string representing the date of birth related data.
/// * `date_of_death`: A string representing the date of death related data.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub date_of_birth: String,
    pub date_of_death: String,
}

/// This function loads the localization data for a staff.
/// It takes a guild_id as input and returns a Result containing StaffLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<StaffLocalised, AppError>`: A Result containing StaffLocalised data or an AppError.
pub async fn load_localization_staff(guild_id: String) -> Result<StaffLocalised, AppError> {
    let path = "json/message/anilist_user/staff.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, StaffLocalised> = serde_json::from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse staff.json. {}", e),
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
