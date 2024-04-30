// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// TranscriptLocalised struct represents a transcript's localized data.
/// It contains a field for title.
///
/// # Struct Fields
/// `title`: A String representing the title of the transcript.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranscriptLocalised {
    pub title: String,
}

/// This function loads the localization data for a transcript.
/// It takes a guild_id as input and returns a Result containing TranscriptLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<TranscriptLocalised, AppError>` - A Result type which is either TranscriptLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_transcript(
    guild_id: String,
) -> Result<TranscriptLocalised, AppError> {
    // Open the JSON file and handle any potential errors
    let mut file = File::open("json/message/ai/transcript.json").map_err(|e| {
        AppError::new(
            format!("File transcript.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Initialize a new String to hold the JSON data
    let mut json = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut json).map_err(|e| {
        AppError::new(
            format!("File transcript.json can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, TranscriptLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse transcript.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the transcript based on the language choice
    let localised_text = json_data.get(lang_choice.as_str()).ok_or(AppError::new(
        "Language not found.".to_string(),
        ErrorType::Language,
        ErrorResponseType::Unknown,
    ))?;

    // Return the localized data
    Ok(localised_text.clone())
}