use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::lang_struct::ai::image::ImageLocalised;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PFPServerLocalisedImage {
    pub title: String,
}

pub async fn load_localization_pfp_server_image(
    guild_id: String,
) -> Result<PFPServerLocalisedImage, AppError> {
    let mut file =
        File::open("json/message/general/generate_image_pfp_server.json").map_err(|e| {
            Error(LocalisationFileError(format!(
                "File generate_image_pfp_server.json not found. {}",
                e
            )))
        })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File generate_image_pfp_server.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, PFPServerLocalisedImage> = serde_json::from_str(&json).map_err(|e| {
        Error(LocalisationParsingError(format!(
            "Failing to parse generate_image_pfp_server.json. {}",
            e
        )))
    })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let pfp_server_image_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(pfp_server_image_localised_text.clone())
}
