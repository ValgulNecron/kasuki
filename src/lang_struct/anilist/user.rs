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
pub struct UserLocalised {
    pub manga: String,
    pub anime: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub weeks: String,
    pub days: String,
    pub hours: String,
    pub minutes: String,
}

pub async fn load_localization_user(guild_id: String) -> Result<UserLocalised, AppError> {
    let mut file = File::open("json/message/anilist/user.json")
        .map_err(|_| LocalisationFileError(String::from("File user.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File user.json can't be read.")))?;

    let json_data: HashMap<String, UserLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse user.json.")))?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != String::from("0"));

    let lang_choice = get_guild_langage(guild_id).await;

    let user_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(user_localised_text.clone())
}
