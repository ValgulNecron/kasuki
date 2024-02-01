use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::NoLangageError;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfoLocalised {
    pub title: String,
    pub desc: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
    pub button_add_the_beta_bot: String,
}

pub async fn load_localization_info(guild_id: String) -> Result<InfoLocalised, AppError> {
    let mut file = File::open("json/message/general/info.json")
        .map_err(|_| LocalisationFileError(String::from("File info.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File info.json can't be read.")))?;

    let json_data: HashMap<String, InfoLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse info.json.")))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let info_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(info_localised_text.clone())
}
