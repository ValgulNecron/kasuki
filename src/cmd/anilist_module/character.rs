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

use crate::constant::COLOR;
use crate::function::error_management::common::custom_error;
use crate::structure::anilist::character::struct_autocomplete_character::CharacterPageWrapper;
use crate::structure::anilist::character::struct_character::CharacterWrapper;
use crate::structure::embed::anilist::struct_lang_character::CharacterLocalisedText;
use crate::structure::register::anilist::struct_character_register::RegisterLocalisedCharacter;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(value) = option {
        let localised_text =
            match CharacterLocalisedText::get_character_localised(ctx, command).await {
                Ok(data) => data,
                Err(_) => return,
            };
        let data: CharacterWrapper = if value.parse::<i32>().is_ok() {
            match CharacterWrapper::new_character_by_id(
                value.parse().unwrap(),
                localised_text.clone(),
            )
            .await
            {
                Ok(character_wrapper) => character_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        } else {
            match CharacterWrapper::new_character_by_search(value, localised_text.clone()).await {
                Ok(character_wrapper) => character_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        };
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
                                .color(COLOR)
                                .description(desc)
                                .thumbnail(image)
                                .field(&localised_text.info, info, true)
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
    let characters = RegisterLocalisedCharacter::get_character_register_localised().unwrap();
    let command = command
        .name("character")
        .description("Get information on a character")
        .create_option(|option| {
            let option = option
                .name("name")
                .description("The name of the character")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for character in characters.values() {
                option
                    .name_localized(&character.code, &character.name)
                    .description_localized(&character.code, &character.desc);
            }
            option
        });
    for character in characters.values() {
        command
            .name_localized(&character.code, &character.name)
            .description_localized(&character.code, &character.desc);
    }
    command
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
