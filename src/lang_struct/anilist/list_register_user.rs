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
pub struct ListUserLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

pub async fn load_localization_list_user(guild_id: String) -> Result<ListUserLocalised, AppError> {
    let mut file = File::open("json/message/anilist/list_register_user.json").map_err(|_| {
        LocalisationFileError(String::from("File list_register_user.json not found."))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|_| {
        LocalisationReadError(String::from("File list_register_user.json can't be read."))
    })?;

    let json_data: HashMap<String, ListUserLocalised> =
        serde_json::from_str(&json).map_err(|_| {
            LocalisationParsingError(String::from("Failing to parse list_register_user.json."))
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let list_user_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(list_user_localised_text.clone())
}
