use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::CommandError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLanguageError,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditLocalisedLine {
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditLocalised {
    pub title: String,
    pub credits: Vec<CreditLocalisedLine>,
}

pub async fn load_localization_credit(guild_id: String) -> Result<CreditLocalised, AppError> {
    let mut file = File::open("json/message/general/credit.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File credit.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File credit.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, CreditLocalised> = serde_json::from_str(&json).map_err(|e| {
        Error(LocalisationParsingError(format!(
            "Failing to parse credit.json. {}",
            e
        )))
    })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let credit_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLanguageError(String::from("not found"))))?;

    Ok(credit_localised_text.clone())
}
