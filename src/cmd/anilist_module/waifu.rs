use crate::function::error_management::common::custom_error;
use crate::structure::anilist::character::struct_character::CharacterWrapper;
use crate::structure::embed::anilist::struct_lang_character::CharacterLocalisedText;
use crate::structure::register::anilist::struct_waifu_register::RegisterLocalisedWaifu;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;
    let localised_text =
        match CharacterLocalisedText::get_character_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
    let data = match CharacterWrapper::new_character_by_id(156323, localised_text.clone()).await {
        Ok(character_wrapper) => character_wrapper,
        Err(error) => {
            custom_error(color, ctx, command, &error).await;
            return;
        }
    };
    let color = Colour::FABLED_PINK;

    let name = data.get_name();
    let url = data.get_url();
    let desc = data.get_desc(localised_text.clone());
    let image = data.get_image();

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(name)
                            .url(url)
                            .timestamp(Timestamp::now())
                            .color(color)
                            .description(desc)
                            .thumbnail(image)
                            .color(color)
                    })
                })
        })
        .await
    {
        println!("Error creating slash command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let waifus = RegisterLocalisedWaifu::get_waifu_register_localised().unwrap();
    let command = command
        .name("waifu")
        .description("Give you the best waifu.")
        .create_option(|option| {
            option
                .name("username")
                .description("Username of the discord user you want the waifu of")
                .kind(CommandOptionType::User)
                .required(false);
            for waifu in waifus.values() {
                option
                    .name_localized(&waifu.code, &waifu.name)
                    .description_localized(&waifu.code, &waifu.desc);
            }
            option
        });
    for waifu in waifus.values() {
        command
            .name_localized(&waifu.code, &waifu.name)
            .description_localized(&waifu.code, &waifu.desc);
    }
    command
}
