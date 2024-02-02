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
use crate::lang_struct::ai::image::ImageLocalised;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacterLocalised {
    pub desc: String,
    pub date_of_birth: String,
}

pub async fn load_localization_character(guild_id: String) -> Result<CharacterLocalised, AppError> {
    let mut file = File::open("json/message/anilist/character.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File character.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File character.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, CharacterLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            Error(LocalisationParsingError(format!(
                "Failing to parse character.json. {}",
                e
            )))
        })?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != *"0");

    let lang_choice = get_guild_langage(guild_id).await;

    let character_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(character_localised_text.clone())
}
