use crate::function::error_management::error_not_nsfw::error_not_nsfw;
use crate::function::general::get_nsfw_channel::get_nsfw;
use crate::structure::anilist::media::struct_media::MediaWrapper;
use crate::structure::embed::anilist::struct_lang_anime::AnimeLocalisedText;
use crate::structure::register::anilist::struct_anime_register::RegisterLocalisedAnime;
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
        let localised_text =
            match AnimeLocalisedText::get_anime_localised(color, ctx, command).await {
                Ok(data) => data,
                Err(_) => return,
            };
        let data: MediaWrapper = if value.parse::<i32>().is_ok() {
            match MediaWrapper::new_anime_by_id(value.parse().unwrap(), color, ctx, command).await {
                Ok(media_wrapper) => media_wrapper,
                Err(_) => {
                    return;
                }
            }
        } else {
            match MediaWrapper::new_anime_by_search(value, color, ctx, command).await {
                Ok(media_wrapper) => media_wrapper,
                Err(_) => {
                    return;
                }
            }
        };

        if data.get_nsfw() && !get_nsfw(command, ctx).await {
            error_not_nsfw(color, ctx, command).await;
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
            println!("Error creating slash command: {}", why);
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let animes = RegisterLocalisedAnime::get_anime_register_localised().unwrap();
    let command = command
        .name("anime")
        .description("Info of an anime")
        .create_option(|option| {
            let option = option
                .name("anime_name")
                .description("Name of the anime you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for anime in animes.values() {
                option
                    .name_localized(&anime.code, &anime.option1)
                    .description_localized(&anime.code, &anime.option1_desc);
            }
            option
        });
    for anime in animes.values() {
        command
            .name_localized(&anime.code, &anime.name)
            .description_localized(&anime.code, &anime.desc);
    }
    command
}
