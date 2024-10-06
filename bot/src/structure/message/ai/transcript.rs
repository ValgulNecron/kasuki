// Importing necessary libraries and modules

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// TranscriptLocalised struct represents a transcript's localized data.
/// It contains a field for title.
///
/// # Struct Fields
/// `title`: A String representing the title of the transcript.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct TranscriptLocalised {
    pub title: String,
}

/// This function loads the localization data for a transcript.
/// It takes a guild_id as input and returns a Result containing TranscriptLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<TranscriptLocalised, AppError>` - A Result type which is either TranscriptLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
use anyhow::Result;

pub async fn load_localization_transcript(
    guild_id: String,
    db_config: DbConfig,
) -> Result<TranscriptLocalised> {
    let path = "json/message/ai/transcript.json";

    load_localization(guild_id, path, db_config).await
}
