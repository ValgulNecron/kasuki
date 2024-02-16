use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::activity::activity_error::ActivityError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SendActivityLocalised {
    pub title: String,
    pub desc: String,
}

pub async fn load_localization_send_activity(
    guild_id: String,
) -> Result<SendActivityLocalised, ActivityError> {
    let mut file = File::open("json/message/anilist/send_activity.json")
        .map_err(|e| LocalisationFileError(format!("File send_activity.json not found. {}", e)))?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File send_activity.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, SendActivityLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            Error(LocalisationParsingError(format!(
                "Failing to parse send_activity.json. {}",
                e
            )))
        })?;

    let send_activity_choice = get_guild_langage(guild_id).await;

    let add_activity_localised_text = json_data
        .get(send_activity_choice.as_str())
        .ok_or(Error(NoLanguageError(String::from("not found"))))?;

    Ok(add_activity_localised_text.clone())
}
