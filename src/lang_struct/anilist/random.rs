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
pub struct RandomLocalised {
    pub desc: String,
}

pub async fn load_localization_random(guild_id: String) -> Result<RandomLocalised, AppError> {
    let mut file = File::open("json/message/anilist/random.json")
        .map_err(|_| LocalisationFileError(String::from("File random.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File random.json can't be read.")))?;

    let json_data: HashMap<String, RandomLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse random.json.")))?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != String::from("0"));

    let lang_choice = get_guild_langage(guild_id).await;

    let random_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(random_localised_text.clone())
}
