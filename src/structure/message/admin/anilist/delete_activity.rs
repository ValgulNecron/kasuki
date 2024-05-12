// Importing necessary libraries and modules
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

/// `DeleteActivityLocalised` is a struct that represents a delete activity's localized data.
/// It contains two fields `success` and `success_desc` which are both Strings.
///
/// # Struct Fields
/// `success`: A String representing the success message of the delete activity.
/// `success_desc`: A String representing the success description of the delete activity.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteActivityLocalised {
    pub success: String,
    pub success_desc: String,
}

/// `load_localization_delete_activity` is an asynchronous function that loads the localized data for a delete activity.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `DeleteActivityLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<DeleteActivityLocalised, AppError>` - A Result type which is either DeleteActivityLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_delete_activity(
    guild_id: String,
) -> Result<DeleteActivityLocalised, AppError> {
    let path = "json/message/admin/anilist/delete_activity.json";
    load_localization(guild_id, path).await
}
