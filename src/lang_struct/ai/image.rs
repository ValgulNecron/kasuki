use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::CommandError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLanguageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageLocalised {
    pub title: String,
}

pub async fn load_localization_image(guild_id: String) -> Result<ImageLocalised, AppError> {
    let mut file = File::open("json/message/ai/image.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File image.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File image.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, ImageLocalised> = serde_json::from_str(&json).map_err(|e| {
        Error(LocalisationParsingError(format!(
            "Failing to parse image.json. {}",
            e
        )))
    })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let image_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLanguageError(String::from("not found"))))?;

    Ok(image_localised_text.clone())
}
