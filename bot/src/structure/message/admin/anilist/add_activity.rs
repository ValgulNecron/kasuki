// Importing necessary libraries and modules
use anyhow::Result;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// `AddActivityLocalised` is a struct that represents an add activity's localized data.
/// It contains four fields `success`, `fail`, `fail_desc` and `success_desc` which are all Strings.
///
/// # Struct Fields
/// `success`: A String representing the success message of the add activity.
/// `fail`: A String representing the failure message of the add activity.
/// `fail_desc`: A String representing the failure description of the add activity.
/// `success_desc`: A String representing the success description of the add activity.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct AddActivityLocalised {
    pub success: String,
    pub fail: String,
    pub fail_desc: String,
    pub success_desc: String,
}

/// `load_localization_add_activity` is an asynchronous function that loads the localized data for an add activity.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `AddActivityLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<AddActivityLocalised, AppError>` - A Result type which is either AddActivityLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.

pub async fn load_localization_add_activity(
    guild_id: String,
    db_config: DbConfig,
) -> Result<AddActivityLocalised> {

    let path = "json/message/admin/anilist/add_activity.json";

    load_localization(guild_id, path, db_config).await
}
