use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;use crate::error_management::file_error::FileError::{NotFound, Parsing, Reading};
use crate::error_management::lang_error::LangError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SteamGameInfoLocalised {
    pub field1: String,
    pub field2: String,
    pub field3: String,
    pub field4: String,
    pub field5: String,
    pub field6: String,
    pub field7: String,
    pub free: String,
    pub coming_soon: String,
    pub tba: String,
}

pub async fn load_localization_steam_game_info(
    guild_id: String,
) -> Result<SteamGameInfoLocalised, LangError> {
    let mut file = File::open("json/message/game/steam_game_info.json")
        .map_err(|e| NotFound(format!("File steam_game_info.json not found. {}", e)))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|e| Reading(format!("File steam_game_info.json can't be read. {}", e)))?;

    let json_data: HashMap<String, SteamGameInfoLocalised> = serde_json::from_str(&json)
        .map_err(|e| Parsing(format!("Failing to parse steam_game_info.json. {}", e)))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(LangError::NotFound())?;

    Ok(localised_text.clone())
}
