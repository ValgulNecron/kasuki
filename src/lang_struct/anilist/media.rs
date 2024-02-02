use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub staff_text: String,
}

pub async fn load_localization_media(guild_id: String) -> Result<MediaLocalised, AppError> {
    let mut file = File::open("json/message/anilist/media.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File media.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File media.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, MediaLocalised> = serde_json::from_str(&json).map_err(|e| {
        Error(LocalisationParsingError(format!(
            "Failing to parse media.json. {}",
            e
        )))
    })?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != *"0");

    let lang_choice = get_guild_langage(guild_id).await;

    let media_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(media_localised_text.clone())
}
