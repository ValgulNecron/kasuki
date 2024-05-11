// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// CharacterLocalised struct represents a character's localized data.
/// It contains fields for description and date of birth.
///
/// # Struct Fields
/// `desc`: A String representing the description of the character.
/// `date_of_birth`: A String representing the date of birth of the character.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacterLocalised {
    pub desc: String,
    pub date_of_birth: String,
}

/// This function loads the localization data for a character.
/// It takes a guild_id as input and returns a Result containing CharacterLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<CharacterLocalised, AppError>` - A Result type which is either CharacterLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_character(guild_id: String) -> Result<CharacterLocalised, AppError> {
    // Open the JSON file and handle any potential errors
    let mut file = File::open("json/message/anilist_user/character.json").map_err(|e| {
        AppError::new(
            format!("File character.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Initialize a new String to hold the JSON data
    let mut json = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut json).map_err(|e| {
        AppError::new(
            format!("File character.json can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, CharacterLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse character.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_language(guild_id).await;

    // Retrieve the localized data for the character based on the language choice
    let localised_text = json_data.get(lang_choice.as_str()).ok_or(AppError::new(
        "Language not found.".to_string(),
        ErrorType::Language,
        ErrorResponseType::Unknown,
    ))?;

    // Return the localized data
    Ok(localised_text.clone())
}
