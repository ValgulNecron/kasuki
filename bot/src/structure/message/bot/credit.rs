use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// Represents a localized line of credit.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains a single field `desc` which is a String.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct CreditLocalisedLine {
    pub desc: String,
}

/// Represents the localized credits.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains two fields `title` and `credits` which are a String and a Vector of `CreditLocalisedLine` respectively.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct CreditLocalised {
    pub title: String,
    pub credits: Vec<CreditLocalisedLine>,
}

/// Loads the localized credits.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized credits for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized credits.
///
/// # Returns
///
/// * `Result<CreditLocalised, AppError>` - The localized credits,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
use anyhow::Result;

pub async fn load_localization_credit(
    guild_id: String,
    db_config: DbConfig,
) -> Result<CreditLocalised> {
    let path = "json/message/bot/credit.json";

    load_localization(guild_id, path, db_config).await
}
