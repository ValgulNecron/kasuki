use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::NoLangageError;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingLocalised {
    pub title: String,
    pub desc: String,
}

pub async fn load_localization_ping(guild_id: String) -> Result<PingLocalised, AppError> {
    let mut file = File::open("json/message/general/ping.json")
        .map_err(|_| LocalisationFileError(String::from("File ping.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File ping.json can't be read.")))?;

    let json_data: HashMap<String, PingLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse ping.json.")))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let ping_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(ping_localised_text.clone())
}
