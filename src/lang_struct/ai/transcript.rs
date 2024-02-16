use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::AppError;
use crate::error_management::error_enum::AppError::Error;
use crate::error_management::error_enum::CommandError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLanguageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranscriptLocalised {
    pub title: String,
}

pub async fn load_localization_transcript(
    guild_id: String,
) -> Result<TranscriptLocalised, AppError> {
    let mut file = File::open("json/message/ai/transcript.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File transcript.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File transcript.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, TranscriptLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            Error(LocalisationParsingError(format!(
                "Failing to parse transcript.json. {}",
                e
            )))
        })?;

    let transcript_choice = get_guild_langage(guild_id).await;

    let transcript_localised_text = json_data
        .get(transcript_choice.as_str())
        .ok_or(Error(NoLanguageError(String::from("not found"))))?;

    Ok(transcript_localised_text.clone())
}
