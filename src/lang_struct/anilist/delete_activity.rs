use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::NoLangageError;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteActivityLocalised {
    pub success: String,
    pub success_desc: String,
}

pub async fn load_localization_delete_activity(
    guild_id: String,
) -> Result<DeleteActivityLocalised, AppError> {
    let mut file = File::open("json/message/anilist/delete_activity.json")
        .map_err(|_| LocalisationFileError(String::from("File delete_activity.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|_| {
        LocalisationReadError(String::from("File delete_activity.json can't be read."))
    })?;

    let json_data: HashMap<String, DeleteActivityLocalised> =
        serde_json::from_str(&json).map_err(|_| {
            LocalisationParsingError(String::from("Failing to parse delete_activity.json."))
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let delete_activity_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(delete_activity_localised_text.clone())
}
