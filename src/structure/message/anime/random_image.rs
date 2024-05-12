use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

/// Represents a random image's localized data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains a single field `title` which is a String.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RandomImageLocalised {
    pub title: String,
}

/// Loads the localized data for a random image.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized data for the specified guild's random image.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized data.
///
/// # Returns
///
/// * `Result<RandomImageLocalised, AppError>` - The localized data for the random image,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_random_image(
    guild_id: String,
) -> Result<RandomImageLocalised, AppError> {
    let path = "json/message/anime/random_image.json";
    load_localization(guild_id, path).await

}
