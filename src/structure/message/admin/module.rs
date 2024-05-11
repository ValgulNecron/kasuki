// Importing necessary libraries and modules
use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

/// `ModuleLocalised` is a struct that represents a module's localized data.
/// It contains two fields `on` and `off` which are both Strings.
///
/// # Struct Fields
/// `on`: A String representing the on state of the module.
/// `off`: A String representing the off state of the module.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModuleLocalised {
    pub on: String,
    pub off: String,
}

/// `load_localization_module_activation` is an asynchronous function that loads the localized data for a module.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `ModuleLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<ModuleLocalised, AppError>` - A Result type which is either ModuleLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_module_activation(
    guild_id: String,
) -> Result<ModuleLocalised, AppError> {
    let path = "json/message/admin/module.json";
    let json = read_file_as_string(path)?;
    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, ModuleLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse module.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_language(guild_id).await;

    // Return the localized data for the module or an error if the language is not found.
    Ok(json_data
        .get(lang_choice.as_str())
        .cloned()
        .unwrap_or(json_data.get("en").unwrap().clone()))
}
