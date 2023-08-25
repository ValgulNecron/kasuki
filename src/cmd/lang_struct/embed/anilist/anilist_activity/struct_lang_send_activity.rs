use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::cmd::general_module::function::get_guild_langage::get_guild_langage;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SendActivityLocalisedText {
    pub title: String,
    pub ep: String,
    pub of: String,
    pub end: String,
}

impl SendActivityLocalisedText {
    pub async fn get_send_activity_localised(
        guild_id: String,
    ) -> Result<SendActivityLocalisedText, &'static str> {
        let mut file = match File::open("lang_file/embed/anilist/anime_activity/send_activity.json")
        {
            Ok(file) => file,
            Err(_) => {
                return Err("not found");
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => {
                return Err("not found");
            }
        }

        let json_data: HashMap<String, SendActivityLocalisedText> =
            match serde_json::from_str(&json) {
                Ok(data) => data,
                Err(_) => {
                    return Err("not found");
                }
            };

        let lang_choice = get_guild_langage(guild_id).await;

        return if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            Ok(localised_text.clone())
        } else {
            Err("not found")
        };
    }
}
