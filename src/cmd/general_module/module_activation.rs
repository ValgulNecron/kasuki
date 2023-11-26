use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::model::{Permissions, Timestamp};

use crate::constant::COLOR;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR};
use crate::function::sqls::general::data::{
    get_data_module_activation_status, set_data_module_activation_status,
};
use crate::structure::embed::general::struct_lang_module_activation::ModuleLocalisedText;
use crate::structure::register::general::struct_modules_register::RegisterLocalisedModule;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();

    let localised_text = ModuleLocalisedText::get_module_localised(&guild_id).await?;
    let mut module = "".to_string();
    let mut state = false;
    for option in options {
        if option.name == "module_name" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(module_option) = resolved {
                module = module_option.clone()
            } else {
                module = "".to_string();
            }
        }
        if option.name == "state" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::Boolean(state_option) = resolved {
                state = *state_option
            } else {
                state = false
            }
        }
    }

    match module.as_str() {
        "ANIME" => {
            let row = get_data_module_activation_status(&guild_id).await?;
            let (_, ai_module, _): (Option<String>, Option<bool>, Option<bool>) = row;

            let ai_value = ai_module.unwrap_or(false);

            set_data_module_activation_status(&guild_id, state, ai_value).await?;

            let text = if state {
                &localised_text.on
            } else {
                &localised_text.off
            };

            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title("ANILIST")
                                    // Add a timestamp for the current time
                                    // This also accepts a rfc3339 Timestamp
                                    .timestamp(Timestamp::now())
                                    .color(COLOR)
                                    .description(text)
                            })
                        })
                })
                .await
                .map_err(|_| COMMAND_SENDING_ERROR.clone())
        }
        "AI" => {
            let row = get_data_module_activation_status(&guild_id).await?;
            let (_, _, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;

            let anilist_value = anilist_module.unwrap_or(false);

            set_data_module_activation_status(&guild_id, anilist_value, state).await?;

            let text = if state {
                &localised_text.on
            } else {
                &localised_text.off
            };

            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title("AI")
                                    // Add a timestamp for the current time
                                    // This also accepts a rfc3339 Timestamp
                                    .timestamp(Timestamp::now())
                                    .color(COLOR)
                                    .description(text)
                            })
                        })
                })
                .await
                .map_err(|_| COMMAND_SENDING_ERROR.clone())
        }
        _ => Err(AppError::ModuleError(String::from(
            "This module does not exist.",
        ))),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let modules = RegisterLocalisedModule::get_module_register_localised().unwrap();
    command
        .name("module")
        .description("Turn on and off module.")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .create_option(|option| {
            option
                .name("module_name")
                .description("The name of the module you want to turn on or off")
                .kind(CommandOptionType::String)
                .add_string_choice("AI", "AI")
                .add_string_choice("ANIME", "ANIME")
                .required(true);
            for module in modules.values() {
                option
                    .name_localized(&module.code, &module.option1)
                    .description_localized(&module.code, &module.option1_desc);
            }
            option
        })
        .create_option(|option| {
            option
                .name("state")
                .description("ON or OFF")
                .kind(CommandOptionType::Boolean)
                .required(true);
            for module in modules.values() {
                option
                    .name_localized(&module.code, &module.option2)
                    .description_localized(&module.code, &module.option2_desc);
            }
            option
        });
    for module in modules.values() {
        command
            .name_localized(&module.code, &module.name)
            .description_localized(&module.code, &module.desc);
    }
    command
}

pub async fn check_activation_status(module: &str, guild_id: String) -> Result<bool, AppError> {
    let row: (Option<String>, Option<bool>, Option<bool>) =
        get_data_module_activation_status(&guild_id).await?;

    let (_, ai_module, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;
    Ok(match module {
        "ANILIST" => anilist_module.unwrap_or(true),
        "AI" => ai_module.unwrap_or(true),
        _ => false,
    })
}
