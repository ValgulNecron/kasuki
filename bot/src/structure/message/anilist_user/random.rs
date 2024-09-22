use std::error::Error;

use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

/// RandomLocalised struct represents a random localized data.
/// It contains a field for description.
///
/// # Struct Fields
/// `desc`: A String representing the description of the random item.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct RandomLocalised {
    pub desc: String,
}

/// This function loads the localization data for a random item.
/// It takes a guild_id as input and returns a Result containing RandomLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<RandomLocalised, AppError>` - A Result type which is either RandomLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
use anyhow::{Context, Result};

pub async fn load_localization_random(
    guild_id: String,
    db_config: DbConfig,
) -> Result<RandomLocalised> {

    let path = "json/message/anilist_user/random.json";

    load_localization(guild_id, path, db_config).await
}
