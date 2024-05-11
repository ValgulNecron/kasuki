use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::read_file::read_file_as_string;

/// Represents the localized banner data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains three fields `title`, `no_banner` and `no_banner_title` which are all Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BannerLocalised {
    pub title: String,
    pub no_banner: String,
    pub no_banner_title: String,
}

/// Loads the localized banner data.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized banner data for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized banner data.
///
/// # Returns
///
/// * `Result<BannerLocalised, AppError>` - The localized banner data,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_banner(guild_id: String) -> Result<BannerLocalised, AppError> {
    let path = "json/message/user/banner.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, BannerLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse banner.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_language(guild_id).await;

    // Return the localized data for the language or an error if the language is not found.
    json_data.get(lang_choice.as_str()).cloned().ok_or_else(|| {
        json_data.get("en").unwrap().cloned()
    })
}
