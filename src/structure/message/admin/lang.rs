// Importing necessary libraries and modules
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

/// `LangLocalised` is a struct that represents a language's localized data.
/// It contains two fields `title` and `desc` which are both Strings.
///
/// # Struct Fields
/// `title`: A String representing the title of the language.
/// `desc`: A String representing the description of the language.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LangLocalised {
    pub title: String,
    pub desc: String,
}

/// `load_localization_lang` is an asynchronous function that loads the localized data for a language.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `LangLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<LangLocalised, AppError>` - A Result type which is either LangLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_lang(guild_id: String) -> Result<LangLocalised, AppError> {
    let path = "json/message/admin/lang.json";
    load_localization(guild_id, path).await

}
