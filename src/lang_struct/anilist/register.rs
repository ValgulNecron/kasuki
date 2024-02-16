use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::AppError;
use crate::error_management::error_enum::AppError::Error;
use crate::error_management::error_enum::CommandError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLanguageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalised {
    pub desc: String,
}

pub async fn load_localization_register(guild_id: String) -> Result<RegisterLocalised, AppError> {
    let mut file = File::open("json/message/anilist/register.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File register.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File register.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, RegisterLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            Error(LocalisationParsingError(format!(
                "Failing to parse register.json. {}",
                e
            )))
        })?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != *"0");

    let lang_choice = get_guild_langage(guild_id).await;

    let register_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLanguageError(String::from("not found"))))?;

    Ok(register_localised_text.clone())
}
