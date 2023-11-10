use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::function::error_management::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json,
};
use crate::function::general::get_guild_langage::get_guild_langage;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnimeLocalisedText {
    pub desc_title: String,
    pub desc_part_1: String,
    pub desc_part_2: String,
    pub desc_part_3: String,
    pub desc_part_4: String,
    pub desc_part_5: String,
    pub desc_part_6: String,
    pub desc_part_7: String,
    pub fields_name_1: String,
    pub fields_name_2: String,
    pub error_anime_not_found: String,
}

impl AnimeLocalisedText {
    pub async fn get_anime_localised(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<AnimeLocalisedText, &'static str> {
        let mut file = match File::open("./lang_file/embed/anilist/anime.json") {
            Ok(file) => file,
            Err(_) => {
                error_langage_file_not_found(ctx, command).await;
                return Err("not found");
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => {
                error_cant_read_langage_file(ctx, command).await;
                return Err("not found");
            }
        }

        let json_data: HashMap<String, AnimeLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                error_parsing_langage_json(ctx, command).await;
                return Err("not found");
            }
        };

        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(ctx, command).await;
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