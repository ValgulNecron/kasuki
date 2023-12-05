use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AvatarLocalised {
    pub title: String,
}

pub async fn load_localization_avatar(guild_id: String) -> Result<AvatarLocalised, AppError> {
    let mut file = File::open("json/message/general/avatar.json")
        .map_err(|_| LocalisationFileError(String::from("File avatar.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File avatar.json can't be read.")))?;

    let json_data: HashMap<String, AvatarLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse avatar.json.")))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let avatar_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(avatar_localised_text.clone())
}