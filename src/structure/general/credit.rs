use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credit {
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Locale {
    pub title: String,
    pub credits: Vec<Credit>,
}

pub async fn load_localization_credit(guild_id: String) -> Result<Locale, AppError> {
    let mut file = File::open("json/message/general/credit.json")
        .map_err(|_| LocalisationFileError(String::from("File credit.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File credit.json can't be read.")))?;

    let json_data: HashMap<String, Locale> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse credit.json.")))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let credit_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(credit_localised_text.clone())
}
