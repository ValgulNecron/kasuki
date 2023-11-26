use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    CommonError, LocalisationFileError, LocalisationParsingError, LocalisationReadError,
    NoLangageError,
};
use crate::function::general::get_guild_langage::get_guild_langage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileLocalisedText {
    pub title: String,
    pub error_no_user: String,
    pub user_id: String,
    pub is_bot: String,
    pub public_flag: String,
    pub joined_at: String,
    pub created_at: String,
}

impl ProfileLocalisedText {
    pub async fn get_profile_localised(guild_id: String) -> Result<ProfileLocalisedText, AppError> {
        let mut file = match File::open("./lang_file/embed/general/profile.json") {
            Ok(file) => file,
            Err(_) => {
                return Err(LocalisationFileError(String::from(
                    "File profile.json not found.",
                )));
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => {
                return Err(LocalisationReadError(String::from(
                    "File profile.json can't be read.",
                )));
            }
        }

        let json_data: HashMap<String, ProfileLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                return Err(LocalisationParsingError(String::from(
                    "Failing to parse profile.json.",
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
