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
pub struct AddActivityLocalisedText {
    pub error_no_media: String,
    pub title1: String,
    pub title2: String,
    pub already_added: String,
    pub adding: String,
    pub error_slash_command: String,
}

impl AddActivityLocalisedText {
    pub async fn get_add_activity_localised(
        guild_id: String,
    ) -> Result<AddActivityLocalisedText, AppError> {
        let mut file = File::open("./lang_file/embed/anilist/anime_activity/add_activity.json")
            .map_err(|_| {
                LocalisationFileError(String::from("File add_activity.json not found."))
            })?;

        let mut json = String::new();
        file.read_to_string(&mut json).map_err(|_| {
            LocalisationReadError(String::from("File add_activity.json can't be read."))
        })?;

        let json_data: HashMap<String, AddActivityLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| {
                LocalisationParsingError(String::from("Failing to parse add_activity.json."))
            })?;

        let lang_choice = get_guild_langage(&guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
