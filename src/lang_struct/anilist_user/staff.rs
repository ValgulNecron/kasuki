use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// StaffLocalised struct represents a staff's localized data.
/// It contains fields for two titles, description, date of birth, and date of death.
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
pub async fn load_localization_staff(guild_id: String) -> Result<StaffLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist_user/staff.json").map_err(|e| {
        AppError::new(
            format!("File staff.json not found or can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, StaffLocalised> = serde_json::from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse staff.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the staff based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
