use std::error::Error;

use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

/// SeiyuuLocalised struct represents a seiyuu's localized data.
/// It contains a field for title.
///
/// # Struct Fields
/// `title`: A String representing the title of the seiyuu.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct SeiyuuLocalised {
    pub title: String,
}

/// This function loads the localization data for a seiyuu.
/// It takes a guild_id as input and returns a Result containing SeiyuuLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<SeiyuuLocalised, AppError>` - A Result type which is either SeiyuuLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.

pub async fn load_localization_seiyuu(
    guild_id: String,
    db_config: DbConfig,
) -> Result<SeiyuuLocalised, Box<dyn Error>> {

    let path = "json/message/anilist_user/seiyuu.json";

    load_localization(guild_id, path, db_config).await
}
