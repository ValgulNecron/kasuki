// Importing necessary libraries and modules

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// ListActivityLocalised struct represents an activity list's localized data.
/// It contains fields for title, next, and previous.
///
/// # Struct Fields
/// `title`: A String representing the title of the activity list.
/// `next`: A String representing the next activity in the list.
/// `previous`: A String representing the previous activity in the list.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ListActivityLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

/// This function loads the localization data for an activity list.
/// It takes a guild_id as input and returns a Result containing ListActivityLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<ListActivityLocalised, AppError>` - A Result type which is either ListActivityLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
use anyhow::Result;

pub async fn load_localization_list_activity(
    guild_id: String,
    db_config: DbConfig,
) -> Result<ListActivityLocalised> {
    let path = "json/message/anilist_server/list_all_activity.json";

    load_localization(guild_id, path, db_config).await
}
