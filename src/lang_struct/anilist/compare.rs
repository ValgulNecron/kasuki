use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use tracing::trace;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompareLocalised {
    pub more_anime: String,
    pub same_anime: String,
    pub more_watch_time: String,
    pub same_watch_time: String,
    pub genre_anime: String,
    pub same_genre_anime: String,
    pub tag_anime: String,
    pub same_tag_anime: String,
    pub more_manga: String,
    pub same_manga: String,
    pub genre_manga: String,
    pub same_genre_manga: String,
    pub tag_manga: String,
    pub same_tag_manga: String,
    pub more_manga_chapter: String,
    pub same_manga_chapter: String,
}

pub async fn load_localization_compare(guild_id: String) -> Result<CompareLocalised, AppError> {
    let mut file = File::open("json/message/anilist/compare.json")
        .map_err(|_| LocalisationFileError(String::from("File compare.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File compare.json can't be read.")))?;

    let json_data: HashMap<String, CompareLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse compare.json.")))?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != String::from("0"));

    let lang_choice = get_guild_langage(guild_id).await;

    let compare_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(compare_localised_text.clone())
}