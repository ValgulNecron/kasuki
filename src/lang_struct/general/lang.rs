use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LangLocalised {
    pub title: String,
    pub desc: String,
}

pub async fn load_localization_lang(guild_id: String) -> Result<LangLocalised, AppError> {
    let mut file = File::open("json/message/general/lang.json")
        .map_err(|_| LocalisationFileError(String::from("File lang_struct.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File lang_struct.json can't be read.")))?;

    let json_data: HashMap<String, LangLocalised> = serde_json::from_str(&json).map_err(|_| {
        LocalisationParsingError(String::from("Failing to parse lang_struct.json."))
    })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let lang_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(lang_localised_text.clone())
}
