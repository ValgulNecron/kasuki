use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::function::error_management::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json, no_langage_error,
};
use crate::function::general::get_guild_langage::get_guild_langage;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AvatarLocalisedText {
    pub title: String,
    pub no_banner_title: String,
    pub error_no_user: String,
}

impl AvatarLocalisedText {
    pub async fn get_avatar_localised(guild_id: String) -> Result<AvatarLocalisedText, AppError> {
        let mut file = match File::open("./lang_file/embed/general/avatar.json") {
            Ok(file) => file,
            Err(_) => {
                return Err(LocalisationFileError(String::from(
                    "File avatar.json not found.",
                )));
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => {
                return Err(LocalisationReadError(String::from(
                    "File avatar.json can't be read.",
                )));
            }
        }

        let json_data: HashMap<String, AvatarLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                return Err(LocalisationParsingError(String::from(
                    "Failing to parse avatar.json.",
                )));
            }
        };

        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            Ok(localised_text.clone())
        } else {
            Err(NoLangageError(String::from("not found")))
        }
    }
}
