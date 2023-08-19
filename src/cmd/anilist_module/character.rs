use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_autocomplete_character::CharacterPageWrapper;
use crate::cmd::anilist_module::struct_character::*;
use crate::cmd::error::common::custom_error;

use crate::cmd::lang_struct::embed::struct_lang_character::CharacterLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;

    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(value) = option {
        let localised_text = match CharacterLocalisedText::get_character_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
            let data: CharacterWrapper;
            if match value.parse::<i32>() {
                Ok(_) => true,
                Err(_) => false,
            } {
                data = match CharacterWrapper::new_character_by_id(
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
                data =
                    match CharacterWrapper::new_character_by_search(value, localised_text.clone())
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => {
                            custom_error(color, ctx, command, &error).await;
                            return;
                        }
                    }
            }
            let name = data.get_name();
            let desc = data.get_desc(localised_text.clone());

            let image = data.get_image();
            let url = data.get_url();

            let info = data.get_info(localised_text.clone());

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
                                    .field(&localised_text.info, info, true)
                                    .color(color)
                            })
                        })
                })
                .await
            {
                println!("{}: {}", "Error creating slash command", why);
            }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("character")
        .description("Get information ")
        .create_option(|option| {
            option
                .name("name")
                .description("The name of the character")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = CharacterPageWrapper::new_autocomplete_character(search, 8).await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}
