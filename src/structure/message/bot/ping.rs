use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

/// Represents the localized ping data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains two fields `title` and `desc` which are both Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PingLocalised {
    pub title: String,
    pub desc: String,
}

/// Loads the localized ping data.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized ping data for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized ping data.
///
/// # Returns
///
/// * `Result<PingLocalised, AppError>` - The localized ping data,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_ping(guild_id: String) -> Result<PingLocalised, AppError> {
    let path = "json/message/bot/ping.json";
    load_localization(guild_id, path).await

}
