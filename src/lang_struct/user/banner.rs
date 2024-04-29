use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `BannerLocalised` is a struct that represents the localized banner.
/// It contains three fields `title`, `no_banner` and `no_banner_title` which are all Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BannerLocalised {
    pub title: String,
    pub no_banner: String,
    pub no_banner_title: String,
}

/// `load_localization_banner` is an asynchronous function that loads the localized banner.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `BannerLocalised` struct or an `AppError`.
///
/// # Errors
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_banner(guild_id: String) -> Result<BannerLocalised, AppError> {
    // Read the JSON file into a String.
    let json = fs::read_to_string("json/message/user/banner.json").map_err(|e| {
        AppError::new(
            format!("File banner.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, BannerLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse banner.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_langage(guild_id).await;

    // Return the localized banner or an error if the language is not found.
    json_data.get(lang_choice.as_str()).cloned().ok_or_else(|| {
        AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        )
    })
}
