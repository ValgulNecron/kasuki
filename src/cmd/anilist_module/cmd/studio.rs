use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::prelude::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::structs::studio::struct_autocomplete_studio::StudioPageWrapper;
use crate::cmd::anilist_module::structs::studio::struct_studio::StudioWrapper;
use crate::cmd::error_module::common::custom_error;
use crate::cmd::lang_struct::embed::anilist::struct_lang_studio::StudioLocalisedText;
use crate::cmd::lang_struct::register::anilist::struct_studio_register::RegisterLocalisedStudio;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let data;
        if match value.parse::<i32>() {
            Ok(_) => true,
            Err(_) => false,
        } {
            data = match StudioWrapper::new_studio_by_id(value.parse().unwrap()).await {
                Ok(studio_wrapper) => studio_wrapper,
                Err(error) => {
                    custom_error(color, ctx, command, &error).await;
                    return;
                }
            }
        } else {
            data = match StudioWrapper::new_studio_by_search(value).await {
                Ok(studio_wrapper) => studio_wrapper,
                Err(error) => {
                    custom_error(color, ctx, command, &error).await;
                    return;
                }
            }
        }
        let localised_text =
            match StudioLocalisedText::get_studio_localised(color, ctx, command).await {
                Ok(data) => data,
                Err(_) => return,
            };
        let name = data.get_studio_name();
        let url = data.get_site_url();
        let color = Colour::FABLED_PINK;
        let desc = data.get_desc(localised_text.clone());

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
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("{}: {}", localised_text.error_slash_command, why);
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let studios = RegisterLocalisedStudio::get_studio_register_localised().unwrap();
    let command = command
        .name("studio")
        .description("Get info on an studio")
        .create_option(|option| {
            let option = option
                .name("studio")
                .description("The name of the studio.")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for (_key, studio) in &studios {
                option
                    .name_localized(&studio.code, &studio.option1)
                    .description_localized(&studio.code, &studio.option1_desc);
            }
            option
        });
    for (_key, studio) in &studios {
        command
            .name_localized(&studio.code, &studio.name)
            .description_localized(&studio.code, &studio.desc);
    }
    command
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = StudioPageWrapper::new_autocomplete_staff(search, 8).await;
        let choices = data.get_choice();
        // doesn't matter if it errors
        let choices_json = json!(choices);
        _ = command
            .create_autocomplete_response(ctx.http.clone(), |response| {
                response.set_choices(choices_json)
            })
            .await;
    }
}
