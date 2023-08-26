use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error_module::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json,
};
use crate::cmd::general_module::function::get_guild_langage::get_guild_langage;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLocalisedText {
    pub manga_title: String,
    pub manga_count: String,
    pub manga_completed: String,
    pub manga_chapter_read: String,
    pub manga_mean_score: String,
    pub manga_standard_deviation: String,
    pub manga_pref_tag: String,
    pub manga_pref_genre: String,
    pub anime_title: String,
    pub anime_count: String,
    pub anime_completed: String,
    pub anime_time_watch: String,
    pub anime_mean_score: String,
    pub anime_standard_deviation: String,
    pub anime_pref_tag: String,
    pub anime_pref_genre: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub error_slash_command: String,
}

impl UserLocalisedText {
    pub async fn get_user_localised(
        color: Colour,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<UserLocalisedText, &'static str> {
        let mut file = match File::open("lang_file/embed/anilist/user.json") {
            Ok(file) => file,
            Err(_) => {
                error_langage_file_not_found(color, ctx, command).await;
                return Err("not found");
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => {
                error_cant_read_langage_file(color, ctx, command).await;
                return Err("not found");
            }
        }

        let json_data: HashMap<String, UserLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                error_parsing_langage_json(color, ctx, command).await;
                return Err("not found");
            }
        };

        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(color, ctx, command).await;
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
