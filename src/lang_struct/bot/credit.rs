use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `CreditLocalisedLine` is a struct that represents a localized line of credit.
/// It contains a single field `desc` which is a String.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreditLocalisedLine {
    pub desc: String,
}

/// `CreditLocalised` is a struct that represents the localized credits.
/// It contains two fields `title` and `credits` which are a String and a Vector of `CreditLocalisedLine` respectively.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreditLocalised {
    pub title: String,
    pub credits: Vec<CreditLocalisedLine>,
}

/// `load_localization_credit` is an asynchronous function that loads the localized credits.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `CreditLocalised` struct or an `AppError`.
///
/// # Errors
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_credit(guild_id: String) -> Result<CreditLocalised, AppError> {
    // Read the JSON file into a String.
    let json = fs::read_to_string("json/message/bot/credit.json").map_err(|e| {
        AppError::new(
            format!("File credit.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Parse the JSON string into a HashMap.
    let json_data: HashMap<String, CreditLocalised> = from_str(&json).map_err(|e| {
        AppError::new(
            format!("Failing to parse credit.json. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Get the language choice based on the guild_id.
    let lang_choice = get_guild_langage(guild_id).await;

    // Return the localized credits or an error if the language is not found.
    json_data.get(lang_choice.as_str()).cloned().ok_or_else(|| {
        AppError::new(
            "Language not found.".to_string(),
            ErrorType::Language,
            ErrorResponseType::Unknown,
        )
    })
}
