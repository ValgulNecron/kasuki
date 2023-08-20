use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::get_nsfw_channel::get_nsfw;
use crate::cmd::anilist_module::struct_media::*;
use crate::cmd::error::common::custom_error;
use crate::cmd::error::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json, no_langage_error,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::MediaLocalisedText;

pub async fn embed(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    search_type: &str,
) {
    let color = Colour::FABLED_PINK;
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let mut file = match File::open("lang_file/embed/anilist/media.json") {
            Ok(file) => file,
            Err(_) => {
                error_langage_file_not_found(color, ctx, command).await;
                return;
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => error_cant_read_langage_file(color, ctx, command).await,
        }

        let json_data: HashMap<String, MediaLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                error_parsing_langage_json(color, ctx, command).await;
                return;
            }
        };

        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(color, ctx, command).await;
                return;
            }
        };
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let data: MediaWrapper;
            if match value.parse::<i32>() {
                Ok(_) => true,
                Err(_) => false,
            } {
                if search_type == "NOVEL" {
                    data = match MediaWrapper::new_ln_by_id(
                        value.parse().unwrap(),
                        localised_text.clone(),
                    )
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => {
                            custom_error(color, ctx, command, &error).await;
                            return;
                        }
                    }
                } else {
                    data = match MediaWrapper::new_manga_by_id(
                        value.parse().unwrap(),
                        localised_text.clone(),
                    )
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => {
                            custom_error(color, ctx, command, &error).await;
                            return;
                        }
                    }
                }
            } else {
                if search_type == "NOVEL" {
                    data = match MediaWrapper::new_ln_by_search(
                        value.parse().unwrap(),
                        localised_text.clone(),
                    )
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => {
                            custom_error(color, ctx, command, &error).await;
                            return;
                        }
                    }
                } else {
                    data = match MediaWrapper::new_manga_by_search(
                        value.parse().unwrap(),
                        localised_text.clone(),
                    )
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => {
                            custom_error(color, ctx, command, &error).await;
                            return;
                        }
                    }
                }
            }

            let is_nsfw = get_nsfw(command, ctx).await;
            if data.data.media.is_adult && !is_nsfw {
                custom_error(color, ctx, command, &localised_text.error_not_nsfw).await;
                return;
            }

            let banner_image = format!("https://img.anili.st/media/{}", data.data.media.id);
            let desc = data.get_desc();
            let thumbnail = data.get_thumbnail();
            let site_url = data.get_url();
            let name = data.get_name();

            let info = data.get_media_info(localised_text.clone());
            let genre = data.get_genres();
            let tag = data.get_tags();
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(name)
                                    .url(site_url)
                                    .timestamp(Timestamp::now())
                                    .color(color)
                                    .description(desc)
                                    .thumbnail(thumbnail)
                                    .image(banner_image)
                                    .field("Info", info, false)
                                    .fields(vec![("Genre", genre, true), ("Tag", tag, true)])
                            })
                        })
                })
                .await
            {
                println!("{}: {}", localised_text.error_slash_command, why);
            }
        } else {
            no_langage_error(color, ctx, command).await
        }
    }
}
