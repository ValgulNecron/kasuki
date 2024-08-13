// Importing necessary libraries and modules

use std::error::Error;

use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// CharacterLocalised struct represents a character's localized data.
/// It contains fields for description and date of birth.
///
/// # Struct Fields
/// `desc`: A String representing the description of the character.
/// `date_of_birth`: A String representing the date of birth of the character.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterLocalised {
    pub date_of_birth: String,

    pub age: String,

    pub gender: String,

    pub fav: String,
    pub blood_type: String,
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
pub async fn load_localization_character(
    guild_id: String,
    db_type: String,
    db_config: BotConfigDetails,
) -> Result<CharacterLocalised, Box<dyn Error>> {
    let path = "json/message/anilist_user/character.json";
    load_localization(guild_id, path, db_type, db_config).await
}
