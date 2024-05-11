use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

/// Represents the localized messages for a new member.
///
/// This struct is used to deserialize the JSON data from the localization file.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewMemberLocalised {
    /// The welcome message for the new member.
    pub welcome: String,
}

/// Loads the localized messages for a new member.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized messages for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized messages.
///
/// # Returns
///
/// * `Result<NewMemberLocalised, AppError>` - The localized messages for the new member,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
pub async fn load_localization_new_member(
    guild_id: String,
) -> Result<NewMemberLocalised, AppError> {
    let path = "json/message/new_member.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, NewMemberLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Error parsing new_member.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_language(guild_id).await;

    // Return the localized data for the language or an error if the language is not found.
    Ok(json_data
        .get(lang_choice.as_str())
        .cloned()
        .unwrap_or(json_data.get("en").unwrap().clone()))
}
