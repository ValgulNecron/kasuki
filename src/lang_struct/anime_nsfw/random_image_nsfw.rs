use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// RandomImageNSFWLocalised struct represents a random NSFW image's localized data.
/// It contains a field for title.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RandomImageNSFWLocalised {
    pub title: String,
}

/// This function loads the localization data for a random NSFW image.
/// It takes a guild_id as input and returns a Result containing RandomImageNSFWLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
pub async fn load_localization_random_image_nsfw(
    guild_id: String,
) -> Result<RandomImageNSFWLocalised, AppError> {
    // Read the JSON file and handle any potential errors
    let json = fs::read_to_string("json/message/anilist_user/random_image_nsfw.json").map_err(|e| {
        AppError::new(
            format!(
                "File random_image_nsfw.json not found or can't be read. {}",
                e
            ),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, RandomImageNSFWLocalised> = serde_json::from_str(&json)
        .map_err(|e| {
            AppError::new(
                format!("Failing to parse random_image_nsfw.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_langage(guild_id).await;

    // Retrieve the localized data for the random NSFW image based on the language choice
    json_data
        .get(lang_choice.as_str())
        .cloned()
        .ok_or(AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        ))
}
