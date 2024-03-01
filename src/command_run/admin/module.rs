use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::command_run::get_option::{get_option_map_bool_subcommand, get_option_map_string_subcommand};
use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::{
    get_data_module_activation_status, set_data_module_activation_status,
};
use crate::database_struct::module_status::ActivationStatusModule;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::general::module::load_localization_module_activation;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand(command_interaction);
    let module = map.get(&String::from("name")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;
    let module_localised = load_localization_module_activation(guild_id.clone()).await?;
    let map = get_option_map_bool_subcommand(command_interaction);
    let state = *map.get(&String::from("state")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    let row = get_data_module_activation_status(&guild_id).await?;
    let mut ai_value = row.ai_module.unwrap_or(true);
    let mut anilist_value = row.anilist_module.unwrap_or(true);
    let mut game_value = row.game_module.unwrap_or(true);
    let mut new_member_value = row.new_member.unwrap_or(true);
    match module.as_str() {
        "ANILIST" => anilist_value = state,
        "AI" => ai_value = state,
        "GAME" => game_value = state,
        "NEW_MEMBER" => new_member_value = state,
        _ => {
            return Err(AppError::new(
                String::from("This module does not exist."),
                ErrorType::Option,
                ErrorResponseType::Message,
            ));
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

pub async fn check_activation_status(module: &str, row: ActivationStatusModule) -> bool {
    match module {
        "ANILIST" => row.anilist_module.unwrap_or(true),
        "AI" => row.ai_module.unwrap_or(true),
        "GAME" => row.game_module.unwrap_or(true),
        "NEW_MEMBER" => row.new_member.unwrap_or(true),
        "ANIME" => row.anime.unwrap_or(true),
        _ => false,
    }
}
