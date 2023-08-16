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
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::AnimeLocalisedText;

// Query made to the anilist api.

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    // Get the content of the first option.
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    // Check if the option variable contain the correct value.
    if let CommandDataOptionValue::String(value) = option {
        let mut file =
            File::open("lang_file/embed/anilist/anime.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, AnimeLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
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
                    Err(error) => return error,
                }
            } else {
                data = match MediaWrapper::new_anime_by_search(value, localised_text.clone()).await
                {
                    Ok(character_wrapper) => character_wrapper,
                    Err(error) => return error,
                }
            }

            if data.get_nsfw() && !get_nsfw(command, ctx).await {
                return localised_text.error_not_nsfw.clone();
            }

            let banner_image = data.get_banner();
            let desc = data.get_desc();
            let thumbnail = data.get_thumbnail();
            let site_url = data.get_url();
            let name = data.get_name();

            let info = data.get_anime_info(localised_text.clone());
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
            return "Language not found".to_string();
        }
    }
    return "good".to_string();
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
