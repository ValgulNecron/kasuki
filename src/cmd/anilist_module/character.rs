use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::CharacterLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(value) = option {
        let mut file = File::open("lang_file/anilist/character.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, CharacterLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let data: CharacterWrapper;
            if match value.parse::<i32>() {
                Ok(_) => true,
                Err(_) => false,
            } {
                data = match CharacterWrapper::new_character_by_id(value.parse().unwrap()).await {
                    Ok(character_wrapper) => character_wrapper,
                    Err(error) => return error,
                }
            } else {
                data = match CharacterWrapper::new_character_by_search(value).await {
                    Ok(character_wrapper) => character_wrapper,
                    Err(error) => return error,
                }
            }

            let color = Colour::FABLED_PINK;

            let name = data.get_name();
            let desc = data.get_desc(localised_text.clone());

            let image = data.get_image();
            let url = data.get_url();

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
                println!("{}: {}", localised_text.error_slash_command ,why);
            }
        }
    }
    return "good".to_string();
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
