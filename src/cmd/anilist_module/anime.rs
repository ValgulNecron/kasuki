use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::get_nsfw_channel::get_nsfw;
use crate::cmd::anilist_module::struct_media::*;
use crate::cmd::error::common::custom_error;
use crate::cmd::error::no_lang_error::{error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id, error_parsing_langage_json, no_langage_error};

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::AnimeLocalisedText;

// Query made to the anilist api.

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    // Get the content of the first option.
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    // Check if the option variable contain the correct value.
    if let CommandDataOptionValue::String(value) = option {
        let color = Colour::FABLED_PINK;
        let mut file = match File::open("lang_file/embed/anilist/anime.json") {
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

        let json_data: HashMap<String, AnimeLocalisedText> = match serde_json::from_str(&json) {
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
                data = match MediaWrapper::new_anime_by_id(
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
                data = match MediaWrapper::new_anime_by_search(value, localised_text.clone()).await
                {
                    Ok(character_wrapper) => character_wrapper,
                    Err(error) => {
                        custom_error(color, ctx, command, &error).await;
                        return;
                    }
                }
            }

            if data.get_nsfw() && !get_nsfw(command, ctx).await {
                custom_error(color, ctx, command, &localised_text.error_not_nsfw).await;
                return;
            }

            let banner_image = data.get_banner();
            let desc = data.get_desc();
            let thumbnail = data.get_thumbnail();
            let site_url = data.get_url();
            let name = data.get_name();

            let info = data.get_anime_info(localised_text.clone());
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
                                    .description(desc)
                                    .thumbnail(thumbnail)
                                    .image(banner_image)
                                    .field(&localised_text.desc_title, info, false)
                                    .fields(vec![
                                        (&localised_text.fields_name_1, genre, true),
                                        (&localised_text.fields_name_2, tag, true),
                                    ])
                                    .color(color)
                            })
                        })
                })
                .await
            {
                println!("{}: {}", &localised_text.error_slash_command, why);
            }
        } else {
            no_langage_error(color, ctx, command).await
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("anime")
        .description("Info of an anime")
        .create_option(|option| {
            option
                .name("anime_name")
                .description("Name of the anime you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}
