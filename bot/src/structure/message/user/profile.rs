use std::error::Error;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// Represents the localized profile data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains two fields `title` and `desc` which are both Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ProfileLocalised {
    pub title: String,
    pub id: String,
    pub creation_date: String,
    pub joined_date: String,
    pub bot: String,
    pub nitro: String,
    pub system: String,
    pub public_flag: String,
    pub premium: String,
}

/// Loads the localized profile data.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized profile data for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized profile data.
///
/// # Returns
///
/// * `Result<ProfileLocalised, AppError>` - The localized profile data,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.

pub async fn load_localization_profile(
    guild_id: String,
    db_config: DbConfig,
) -> Result<ProfileLocalised, Box<dyn Error>> {

    let path = "json/message/user/profile.json";

    load_localization(guild_id, path, db_config).await
}
