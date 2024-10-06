// Importing necessary libraries and modules

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// ListUserLocalised struct represents a user list's localized data.
/// It contains fields for title, next, and previous.
///
/// # Struct Fields
/// `title`: A String representing the title of the user list.
/// `next`: A String representing the next user in the list.
/// `previous`: A String representing the previous user in the list.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ListUserLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

/// This function loads the localization data for a user list.
/// It takes a guild_id as input and returns a Result containing ListUserLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<ListUserLocalised, AppError>` - A Result type which is either ListUserLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
use anyhow::Result;

pub async fn load_localization_list_user(
    guild_id: String,
    db_config: DbConfig,
) -> Result<ListUserLocalised> {
    let path = "json/message/anilist_server/list_register_user.json";

    load_localization(guild_id, path, db_config).await
}
