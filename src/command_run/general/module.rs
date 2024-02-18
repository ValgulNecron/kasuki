use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::{
    get_data_module_activation_status, set_data_module_activation_status,
};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::general::module::load_localization_module_activation;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
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
                module = "".to_string()
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

    let row = get_data_module_activation_status(&guild_id).await?;
    let (_, ai_module, anilist_module, game_module, new_member_value): (
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    ) = row;
    let mut ai_value = ai_module.unwrap_or(true);
    let mut anilist_value = anilist_module.unwrap_or(true);
    let mut game_value = game_module.unwrap_or(true);
    let mut new_member_value = new_member_value.unwrap_or(true);
    match module.as_str() {
        "ANIME" => anilist_value = state,
        "AI" => ai_value = state,
        "GAME" => game_value = state,
        "NEW MEMBER" => new_member_value = state,
        _ => {
            return Err(
                AppError::new(
                    String::from("This module does not exist."),
                    ErrorType::Option,
                    ErrorResponseType::Message,
                )
            );
        }
    }
    set_data_module_activation_status(
        &guild_id,
        anilist_value,
        ai_value,
        game_value,
        new_member_value,
    )
    .await?;
    let desc = if state {
        &module_localised.on
    } else {
        &module_localised.off
    };

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(module);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}

pub async fn check_activation_status(
    module: &str,
    guild_id: String,
) -> Result<bool, CommandInteraction> {
    let row: (
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    ) = get_data_module_activation_status(&guild_id).await?;

    let (_, ai_module, anilist_module, game_module, new_member): (
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    ) = row;
    Ok(match module {
        "ANILIST" => anilist_module.unwrap_or(true),
        "AI" => ai_module.unwrap_or(true),
        "GAME" => game_module.unwrap_or(true),
        "NEW_MEMBER" => new_member.unwrap_or(true),
        _ => false,
    })
}
