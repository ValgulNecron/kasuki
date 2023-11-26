use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    CommonError, LocalisationFileError, LocalisationParsingError, LocalisationReadError,
};
use crate::function::general::get_guild_langage::get_guild_langage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PingLocalisedText {
    pub title: String,
    pub description_part_1: String,
    pub description_part_2: String,
    pub description_part_3: String,
}

impl PingLocalisedText {
    pub async fn get_ping_localised(guild_id: String) -> Result<PingLocalisedText, AppError> {
        let mut file = match File::open("./lang_file/embed/general/ping.json") {
            Ok(file) => file,
            Err(_) => {
                return Err(LocalisationFileError(String::from(
                    "File ping.json not found.",
                )));
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => {
                return Err(LocalisationReadError(String::from(
                    "File ping.json can't be read.",
                )));
            }
        }

        let json_data: HashMap<String, PingLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                return Err(LocalisationParsingError(String::from(
                    "Failing to parse ping.json.",
                )));
            }
        };

        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            Ok(localised_text.clone())
        } else {
            Err(CommonError(String::from("not found")))
        }
    }
}
