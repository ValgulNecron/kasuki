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
pub struct ListActivityLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

pub async fn load_localization_list_activity(
    guild_id: String,
) -> Result<ListActivityLocalised, AppError> {
    let mut file = File::open("json/message/anilist/list_all_activity.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File list_all_activity.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File list_all_activity.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, ListActivityLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            Error(LocalisationParsingError(format!(
                "Failing to parse list_all_activity.json. {}",
                e
            )))
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let list_activity_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLanguageError(String::from("not found"))))?;

    Ok(list_activity_localised_text.clone())
}
