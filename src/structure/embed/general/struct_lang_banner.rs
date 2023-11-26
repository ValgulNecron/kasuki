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
pub struct BannerLocalisedText {
    pub error_slash_command: String,
    pub title: String,
    pub description: String,
    pub no_banner_title: String,
    pub error_no_user: String,
}

impl BannerLocalisedText {
    pub async fn get_banner_localised(guild_id: String) -> Result<BannerLocalisedText, AppError> {
        let mut file = File::open("./lang_file/embed/general/banner.json")
            .map_err(|_| LocalisationFileError(String::from("File banner.json not found.")))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| LocalisationReadError(String::from("File banner.json can't be read.")))?;

        let json_data: HashMap<String, BannerLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| LocalisationParsingError(String::from("Failing to parse banner.json.")))?;

        let lang_choice = get_guild_langage(&guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
