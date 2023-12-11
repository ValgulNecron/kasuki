use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::lang_struct::general::module::load_localization_module_activation;
use crate::sqls::general::data::{
    get_data_module_activation_status, set_data_module_activation_status,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let module_localised = load_localization_module_activation(guild_id.clone()).await?;
    let mut module = "".to_string();
    let mut state = false;
    for option in options {
        if option.name == "module_name" {
            let resolved = &option.value;
            if let CommandDataOptionValue::String(module_option) = resolved {
                module = module_option.clone()
            } else {
                module = "".to_string();
            }
        }
        if option.name == "state" {
            let resolved = &option.value;
            if let CommandDataOptionValue::Boolean(state_option) = resolved {
                state = *state_option
            } else {
                state = false
            }
        }
    }

    let desc;
    match module.as_str() {
        "ANIME" => {
            let row = get_data_module_activation_status(&guild_id).await?;
            let (_, ai_module, _): (Option<String>, Option<bool>, Option<bool>) = row;

            let ai_value = ai_module.unwrap_or(false);

            set_data_module_activation_status(&guild_id, state, ai_value).await?;

            desc = if state {
                &module_localised.on
            } else {
                &module_localised.off
            };
        }
        "AI" => {
            let row = get_data_module_activation_status(&guild_id).await?;
            let (_, _, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;

            let anilist_value = anilist_module.unwrap_or(false);

            set_data_module_activation_status(&guild_id, anilist_value, state).await?;

            desc = if state {
                &module_localised.on
            } else {
                &module_localised.off
            };
        }
        _ => {
            return Err(AppError::ModuleError(String::from(
                "This module does not exist.",
            )))
        }
    }

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(module);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
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
