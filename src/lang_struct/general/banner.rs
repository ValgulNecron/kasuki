use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BannerLocalised {
    pub title: String,
    pub no_banner: String,
    pub no_banner_title: String,
}

pub async fn load_localization_banner(guild_id: String) -> Result<BannerLocalised, AppError> {
    let mut file = File::open("json/message/general/banner.json")
        .map_err(|_| LocalisationFileError(String::from("File image.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File image.json can't be read.")))?;

    let json_data: HashMap<String, BannerLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse image.json.")))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let banner_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(banner_localised_text.clone())
}
