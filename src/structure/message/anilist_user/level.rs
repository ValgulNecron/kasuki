use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;

/// LevelLocalised struct represents a level's localized data.
/// It contains a field for description.
///
/// # Struct Fields
/// `desc`: A String representing the description of the level.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LevelLocalised {
    pub desc: String,
}

/// This function loads the localization data for a level.
/// It takes a guild_id as input and returns a Result containing LevelLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<LevelLocalised, AppError>` - A Result type which is either LevelLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_level(
    guild_id: String,
    db_type: String,
) -> Result<LevelLocalised, AppError> {
    let path = "json/message/anilist_user/level.json";
    load_localization(guild_id, path, db_type).await
}
