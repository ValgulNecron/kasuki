use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::function::general::get_guild_langage::get_guild_langage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InfoLocalisedText {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
    pub server_specific_info: String,
    pub on: String,
    pub off: String,
}

impl InfoLocalisedText {
    pub async fn get_info_localised(guild_id: String) -> Result<InfoLocalisedText, AppError> {
        let mut file = File::open("./lang_file/embed/general/info.json")
            .map_err(|_| LocalisationFileError(String::from("File info.json not found.")))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| LocalisationReadError(String::from("File info.json can't be read.")))?;

        let json_data: HashMap<String, InfoLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| LocalisationParsingError(String::from("Failing to parse info.json.")))?;

        let lang_choice = get_guild_langage(&guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
