use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use tracing::trace;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub staff_text: String,
}

pub async fn load_localization_media(guild_id: String) -> Result<MediaLocalised, AppError> {
    let mut file = File::open("json/message/anilist/media.json")
        .map_err(|_| LocalisationFileError(String::from("File media.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File media.json can't be read.")))?;

    let json_data: HashMap<String, MediaLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse media.json.")))?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != String::from("0"));

    let lang_choice = if guild_id != String::from("0") {
        get_guild_langage(guild_id).await
    } else {
        String::from("en")
    };

    let media_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(media_localised_text.clone())
}
