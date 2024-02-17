use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;use crate::error_management::file_error::FileError::{NotFound, Parsing, Reading};
use crate::error_management::lang_error::LangError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompareLocalised {
    pub affinity: String,
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

pub async fn load_localization_compare(guild_id: String) -> Result<CompareLocalised, LangError> {
    let mut file = File::open("json/message/anilist/compare.json")
        .map_err(|e| NotFound(format!("File compare.json not found. {}", e)))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|e| Reading(format!("File compare.json can't be read. {}", e)))?;

    let json_data: HashMap<String, CompareLocalised> = serde_json::from_str(&json)
        .map_err(|e| Parsing(format!("Failing to parse compare.json. {}", e)))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(LangError::NotFound())?;

    Ok(localised_text.clone())
}
