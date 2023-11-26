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
        let mut file = File::open("./lang_file/embed/general/profile.json")
            .map_err(|_| LocalisationFileError(String::from("File profile.json not found.")))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| LocalisationReadError(String::from("File profile.json can't be read.")))?;

        let json_data: HashMap<String, ProfileLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| {
                LocalisationParsingError(String::from("Failing to parse profile.json."))
            })?;

        let lang_choice = get_guild_langage(guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
