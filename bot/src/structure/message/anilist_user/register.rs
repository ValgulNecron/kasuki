use std::error::Error;

use crate::config::BotConfigDetails;
use serde::{Deserialize, Serialize};
// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

/// RegisterLocalised struct represents a register's localized data.
/// It contains a field for description.
///
/// # Struct Fields
/// `desc`: A String representing the description of the register.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalised {
    pub desc: String,
}

/// This function loads the localization data for a register.
/// It takes a guild_id as input and returns a Result containing RegisterLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<RegisterLocalised, AppError>` - A Result type which is either RegisterLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_register(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<RegisterLocalised, Box<dyn Error>> {
    let path = "json/message/anilist_user/register.json";
    load_localization(guild_id, path, db_config).await
}
