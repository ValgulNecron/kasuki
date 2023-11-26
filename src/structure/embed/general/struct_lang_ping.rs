use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::function::general::get_guild_langage::get_guild_langage;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PingLocalisedText {
    pub title: String,
    pub description_part_1: String,
    pub description_part_2: String,
    pub description_part_3: String,
}

impl PingLocalisedText {
    pub async fn get_ping_localised(guild_id: String) -> Result<PingLocalisedText, AppError> {
        let mut file = File::open("./lang_file/embed/general/ping.json")
            .map_err(|_| LocalisationFileError(String::from("File ping.json not found.")))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| LocalisationReadError(String::from("File ping.json can't be read.")))?;

        let json_data: HashMap<String, PingLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| LocalisationParsingError(String::from("Failing to parse ping.json.")))?;

        let lang_choice = get_guild_langage(&guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
