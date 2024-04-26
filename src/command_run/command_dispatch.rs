use serde::de::Unexpected::Str;
use serenity::all::{CommandInteraction, Context, ResolvedValue};
use tracing::trace;

use crate::command_run::admin::module::check_activation_status;
use crate::command_run::admin::{lang, module};
use crate::command_run::ai::{image, question, transcript, translation};
use crate::command_run::anilist::{
    add_activity, anime, character, compare, delete_activity, level, list_all_activity,
    list_register_user, ln, manga, random, register, search, seiyuu, staff, studio, user, waifu,
};
use crate::command_run::anime::{random_image, random_nsfw_image};
use crate::command_run::bot_info::{credit, info, ping};
use crate::command_run::general::{
    avatar, banner, generate_image_pfp_server, generate_image_pfp_server_global, guild, profile,
};
use crate::command_run::steam::steam_game_info;
use crate::common::get_option::subcommand_group::get_subcommand;
use crate::database::dispatcher::data_dispatch::{
    get_data_module_activation_kill_switch_status, get_data_module_activation_status,
};
use crate::database_struct::module_status::ActivationStatusModule;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn command_dispatching(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let ai_module_error: AppError = AppError {
        message: String::from("AI module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    let anilist_module_error: AppError = AppError {
        message: String::from("Anilist module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    let game_module_error: AppError = AppError {
        message: String::from("Game module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    let command_name = command_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str();
    match command_interaction.data.name.as_str() {
        // admin module
        "admin" => admin(ctx, command_interaction, command_name).await?,

        _ => {
            return Err(AppError::new(
                String::from("Command does not exist."),
                ErrorType::Option,
                ErrorResponseType::Message,
            ));
        }
    }

    Ok(())
}

pub async fn check_if_module_is_on(guild_id: String, module: &str) -> Result<bool, AppError> {
    let row: ActivationStatusModule = get_data_module_activation_status(&guild_id).await?;
    let state = check_activation_status(module, row).await;
    let state = state && check_kill_switch_status(module).await?;
    Ok(state)
}

async fn check_kill_switch_status(module: &str) -> Result<bool, AppError> {
    let row: ActivationStatusModule = get_data_module_activation_kill_switch_status().await?;
    Ok(check_activation_status(module, row).await)
}

async fn admin(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    let anime_module_error: AppError = AppError {
        message: String::from("Anime module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    match command_name {
        "lang" => lang::run(ctx, command_interaction).await,
        "module" => module::run(ctx, command_interaction).await,
        "anilist" => {
            if check_if_module_is_on(guild_id, "ANIME").await? {
                let subcommand = get_subcommand(command_interaction).unwrap();
                trace!("{:#?}", subcommand);
                let subcommand_name = subcommand.name;
                anilist_admin(ctx, command_interaction, subcommand_name).await
            } else {
                return Err(anime_module_error.clone());
            }
        }
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn anilist_admin(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "add_anime_activity" => add_activity::run(ctx, command_interaction).await,
        "delete_activity" => delete_activity::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn ai(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "image" => image::run(ctx, command_interaction).await,
        "transcript" => transcript::run(ctx, command_interaction).await,
        "translation" => translation::run(ctx, command_interaction).await,
        "question" => question::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn anime(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "random_image" => random_image::run(ctx, command_interaction).await,
        "random_nsfw_image" => random_nsfw_image::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn steam(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "game" => steam_game_info::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn bot_info(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "credit" => credit::run(ctx, command_interaction).await,
        "info" => info::run(ctx, command_interaction).await,
        "ping" => ping::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn general(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "avatar" => avatar::run(ctx, command_interaction).await,
        "banner" => banner::run(ctx, command_interaction).await,
        "profile" => profile::run(ctx, command_interaction).await,
        "guild" => guild::run(ctx, command_interaction).await,
        "guild_image" => generate_image_pfp_server::run(ctx, command_interaction).await,
        "list_activity" => list_all_activity::run(ctx, command_interaction).await,
        "guild_image_g" => generate_image_pfp_server_global::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn anilist(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "anime" => anime::run(ctx, command_interaction).await,
        "ln" => ln::run(ctx, command_interaction).await,
        "manga" => manga::run(ctx, command_interaction).await,

        "user" => user::run(ctx, command_interaction).await,
        "character" => character::run(ctx, command_interaction).await,
        "waifu" => waifu::run(ctx, command_interaction).await,
        "compare" => compare::run(ctx, command_interaction).await,
        "random" => random::run(ctx, command_interaction).await,
        "register" => register::run(ctx, command_interaction).await,
        "staff" => staff::run(ctx, command_interaction).await,
        "studio" => studio::run(ctx, command_interaction).await,
        "search" => search::run(ctx, command_interaction).await,
        "seiyuu" => seiyuu::run(ctx, command_interaction).await,
        "level" => level::run(ctx, command_interaction).await,
        "list_user" => list_register_user::run(ctx, command_interaction).await,
        "random_image" => random_image::run(ctx, command_interaction).await,
        "random_nsfw_image" => random_nsfw_image::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}
