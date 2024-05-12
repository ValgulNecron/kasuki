// Importing necessary libraries and modules
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

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
    let path = "json/message/anilist_user/character.json";
    load_localization(guild_id, path).await

}
