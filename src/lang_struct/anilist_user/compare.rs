// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::helper::get_guild_lang::get_guild_language;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// CompareLocalised struct represents a comparison's localized data.
/// It contains fields for affinity, more_anime, same_anime, more_watch_time, same_watch_time, genre_anime, same_genre_anime, tag_anime, same_tag_anime, more_manga, same_manga, genre_manga, same_genre_manga, tag_manga, same_tag_manga, more_manga_chapter, same_manga_chapter.
///
/// # Struct Fields
/// `affinity`: A String representing the affinity of the comparison.
/// `more_anime`: A String representing the more_anime of the comparison.
/// `same_anime`: A String representing the same_anime of the comparison.
/// `more_watch_time`: A String representing the more_watch_time of the comparison.
/// `same_watch_time`: A String representing the same_watch_time of the comparison.
/// `genre_anime`: A String representing the genre_anime of the comparison.
/// `same_genre_anime`: A String representing the same_genre_anime of the comparison.
/// `tag_anime`: A String representing the tag_anime of the comparison.
/// `same_tag_anime`: A String representing the same_tag_anime of the comparison.
/// `more_manga`: A String representing the more_manga of the comparison.
/// `same_manga`: A String representing the same_manga of the comparison.
/// `genre_manga`: A String representing the genre_manga of the comparison.
/// `same_genre_manga`: A String representing the same_genre_manga of the comparison.
/// `tag_manga`: A String representing the tag_manga of the comparison.
/// `same_tag_manga`: A String representing the same_tag_manga of the comparison.
/// `more_manga_chapter`: A String representing the more_manga_chapter of the comparison.
/// `same_manga_chapter`: A String representing the same_manga_chapter of the comparison.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompareLocalised {
    pub affinity: String,
    pub more_anime: String,
    pub same_anime: String,
    pub more_watch_time: String,
    pub same_watch_time: String,
    pub genre_anime: String,
    pub same_genre_anime: String,
    pub tag_anime: String,
    pub same_tag_anime: String,
    pub more_manga: String,
    pub same_manga: String,
    pub genre_manga: String,
    pub same_genre_manga: String,
    pub tag_manga: String,
    pub same_tag_manga: String,
    pub more_manga_chapter: String,
    pub same_manga_chapter: String,
}

/// This function loads the localization data for a comparison.
/// It takes a guild_id as input and returns a Result containing CompareLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<CompareLocalised, AppError>` - A Result type which is either CompareLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_compare(guild_id: String) -> Result<CompareLocalised, AppError> {
    // Open the JSON file and handle any potential errors
    let mut file = File::open("json/message/anilist_user/compare.json").map_err(|e| {
        AppError::new(
            format!("File compare.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Initialize a new String to hold the JSON data
    let mut json = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut json).map_err(|e| {
        AppError::new(
            format!("File compare.json can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, CompareLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse compare.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Get the language choice for the guild
    let lang_choice = get_guild_language(guild_id).await;

    // Retrieve the localized data for the comparison based on the language choice
    let localised_text = json_data.get(lang_choice.as_str()).ok_or(AppError::new(
        "Language not found.".to_string(),
        ErrorType::Language,
        ErrorResponseType::Unknown,
    ))?;

    // Return the localized data
    Ok(localised_text.clone())
}
