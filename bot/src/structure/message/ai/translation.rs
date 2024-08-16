// Importing necessary libraries and modules

use std::error::Error;

use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// TranslationLocalised struct represents a translation's localized data.
/// It contains a field for title.
///
/// # Struct Fields
/// `title`: A String representing the title of the translation.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranslationLocalised {
    pub title: String,
}

/// This function loads the localization data for a translation.
/// It takes a guild_id as input and returns a Result containing TranslationLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<TranslationLocalised, AppError>` - A Result type which is either TranslationLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_translation(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<TranslationLocalised, Box<dyn Error>> {
    let path = "json/message/ai/translation.json";
    load_localization(guild_id, path, db_config).await
}
