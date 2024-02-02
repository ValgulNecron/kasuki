use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::lang_struct::ai::image::ImageLocalised;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddActivityLocalised {
    pub success: String,
    pub fail: String,
    pub fail_desc: String,
    pub success_desc: String,
}

pub async fn load_localization_add_activity(
    guild_id: String,
) -> Result<AddActivityLocalised, AppError> {
    let mut file = File::open("json/message/anilist/add_activity.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File add_activity.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File add_activity.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, AddActivityLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            Error(LocalisationParsingError(format!(
                "Failing to parse add_activity.json. {}",
                e
            )))
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let add_activity_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(add_activity_localised_text.clone())
}
