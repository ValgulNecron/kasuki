use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileLocalised {
    pub title: String,
    pub desc: String,
}

pub async fn load_localization_profile(guild_id: String) -> Result<ProfileLocalised, AppError> {
    let mut file = File::open("json/message/general/profile.json")
        .map_err(|_| LocalisationFileError(String::from("File profile.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File profile.json can't be read.")))?;

    let json_data: HashMap<String, ProfileLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse profile.json.")))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let profile_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(profile_localised_text.clone())
}
