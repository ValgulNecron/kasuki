use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::command_error::CommandError;
use crate::error_management::file_error::FileError::{NotFound, Parsing, Reading};
use crate::error_management::lang_error::LangError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StudioLocalised {
    pub desc: String,
}

pub async fn load_localization_studio(guild_id: String) -> Result<StudioLocalised, CommandError> {
    let mut file = File::open("json/message/anilist/studio.json")
        .map_err(|e| NotFound(format!("File studio.json not found. {}", e)))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|e| Reading(format!("File studio.json can't be read. {}", e)))?;

    let json_data: HashMap<String, StudioLocalised> = serde_json::from_str(&json)
        .map_err(|e| Parsing(format!("Failing to parse studio.json. {}", e)))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(LangError::NotFound())?;

    Ok(localised_text.clone())
}
