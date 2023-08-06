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
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::MediaLocalisedText;

pub async fn embed(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    search_type: &str,
) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let mut file = File::open("lang_file/anilist/media.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, MediaLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
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
                        Err(error) => return error,
                    }
                } else {
                    data = match MediaWrapper::new_manga_by_id(
                        value.parse().unwrap(),
                        localised_text.clone(),
                    )
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => return error,
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
                        Err(error) => return error,
                    }
                } else {
                    data = match MediaWrapper::new_manga_by_search(
                        value.parse().unwrap(),
                        localised_text.clone(),
                    )
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => return error,
                    }
                }
            }

            let is_nsfw = get_nsfw(command, ctx).await;
            if data.data.media.is_adult && !is_nsfw {
                return "not an NSFW channel".to_string();
            }

            let banner_image = format!("https://img.anili.st/media/{}", data.data.media.id);
            let desc = data.get_desc();
            let thumbnail = data.get_thumbnail();
            let site_url = data.get_url();
            let name = data.get_name();

            let info = data.get_media_info(localised_text.clone());
            let genre = data.get_genres();
            let tag = data.get_tags();

            let color = Colour::FABLED_PINK;

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
                                    .color(color)
                            })
                        })
                })
                .await
            {
                println!("{}: {}", localised_text.error_slash_command, why);
            }
        } else {
            return "Language not found".to_string();
        }
    }
    return "good".to_string();
}
