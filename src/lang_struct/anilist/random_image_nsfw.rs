use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::NoLangageError;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RandomImageNSFWLocalised {
    pub title: String,
}

pub async fn load_localization_random_image_nsfw(
    guild_id: String,
) -> Result<RandomImageNSFWLocalised, AppError> {
    let mut file = File::open("json/message/anilist/random_image_nsfw.json").map_err(|_| {
        LocalisationFileError(String::from("File random_image_nsfw.json not found."))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|_| {
        LocalisationReadError(String::from("File random_image_nsfw.json can't be read."))
    })?;

    let json_data: HashMap<String, RandomImageNSFWLocalised> = serde_json::from_str(&json)
        .map_err(|_| {
            LocalisationParsingError(String::from("Failing to parse random_image_nsfw.json."))
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let random_image_nsfw_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(random_image_nsfw_localised_text.clone())
}
