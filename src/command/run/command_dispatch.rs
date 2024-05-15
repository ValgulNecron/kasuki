use serenity::all::{CommandInteraction, Context};
use tracing::trace;

use crate::command::run::admin::anilist::{add_activity, delete_activity};
use crate::command::run::admin::module::check_activation_status;
use crate::command::run::admin::{lang, module};
use crate::command::run::ai::{image, question, transcript, translation};
use crate::command::run::anilist_server::{list_all_activity, list_register_user};
use crate::command::run::anilist_user::{
    anime, character, compare, level, ln, manga, random, register, search, seiyuu, staff, studio,
    user, waifu,
};
use crate::command::run::anime::random_image;
use crate::command::run::anime_nsfw::random_nsfw_image;
use crate::command::run::bot::{credit, info, ping};
use crate::command::run::server::{
    generate_image_pfp_server, generate_image_pfp_server_global, guild,
};
use crate::command::run::steam::steam_game_info;
use crate::command::run::user::{avatar, banner, profile};
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::manage::dispatcher::data_dispatch::{
    get_data_module_activation_kill_switch_status, get_data_module_activation_status,
};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand_group::get_subcommand;

/// Dispatches the command to the appropriate function based on the command name.
///
/// This function retrieves the command name from the command interaction and matches it to the appropriate function.
/// If the command name does not match any of the specified commands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
pub async fn command_dispatching(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    // Retrieve the command name from the command interaction
    let command_name = command_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str();
    // Match the command name to the appropriate function
    match command_interaction.data.name.as_str() {
        // anilist_user module
        "admin" => admin(ctx, command_interaction, command_name).await?,
        "ai" => ai(ctx, command_interaction, command_name).await?,
        "anilist_server" => anilist_server(ctx, command_interaction, command_name).await?,
        "anilist_user" => anilist_user(ctx, command_interaction, command_name).await?,
        "anime" => anime(ctx, command_interaction, command_name).await?,
        "anime_nsfw" => anime_nsfw(ctx, command_interaction, command_name).await?,
        "bot" => bot(ctx, command_interaction, command_name).await?,
        "server" => server(ctx, command_interaction, command_name).await?,
        "steam" => steam(ctx, command_interaction, command_name).await?,
        "user" => user(ctx, command_interaction, command_name).await?,
        "vn" => vn(ctx, command_interaction, command_name).await?,
        // If the command name does not match any of the specified commands, return an error
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

/// Checks if a module is activated.
///
/// This function retrieves the activation status of a module for a specific guild.
/// It checks both the activation status and the kill switch status of the module.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild.
/// * `module` - The name of the module.
///
/// # Returns
///
/// A `Result` that is `Ok` if the module is activated, or `Err` if an error occurred.
pub async fn check_if_module_is_on(guild_id: String, module: &str) -> Result<bool, AppError> {
    let row: ActivationStatusModule = get_data_module_activation_status(&guild_id).await?;
    let state = check_activation_status(module, row).await;
    let state = state && check_kill_switch_status(module).await?;
    Ok(state)
}

/// Checks the kill switch status of a module.
///
/// This function retrieves the kill switch status of a module.
///
/// # Arguments
///
/// * `module` - The name of the module.
///
/// # Returns
///
/// A `Result` that is `Ok` if the kill switch is not activated, or `Err` if an error occurred.
async fn check_kill_switch_status(module: &str) -> Result<bool, AppError> {
    let row: ActivationStatusModule = get_data_module_activation_kill_switch_status().await?;
    Ok(check_activation_status(module, row).await)
}

/// Executes the admin command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
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
    trace!(command_name);
    match command_name {
        "general" => {
            let subcommand = get_subcommand(command_interaction).unwrap();
            let subcommand_name = subcommand.name;
            general_admin(ctx, command_interaction, subcommand_name).await
        }
        "anilist" => {
            if check_if_module_is_on(guild_id, "ANIME").await? {
                let subcommand = get_subcommand(command_interaction).unwrap();
                trace!("{:#?}", subcommand);
                let subcommand_name = subcommand.name;
                anilist_admin(ctx, command_interaction, subcommand_name).await
            } else {
                Err(anime_module_error.clone())
            }
        }
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the admin command for the Anilist module.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
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

/// Executes the general admin command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn general_admin(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "lang" => lang::run(ctx, command_interaction).await,
        "module" => module::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the AI command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
/// It also checks if the AI module is activated for the guild. If not, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn ai(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    // Define the error for when the AI module is off
    let ai_module_error: AppError = AppError {
        message: String::from("AI module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the AI module is on for the guild
    if !check_if_module_is_on(guild_id, "AI").await? {
        return Err(ai_module_error);
    }
    // Match the command name to the appropriate function
    match command_name {
        "image" => image::run(ctx, command_interaction).await,
        "transcript" => transcript::run(ctx, command_interaction).await,
        "translation" => translation::run(ctx, command_interaction).await,
        "question" => question::run(ctx, command_interaction).await,
        // If the command name does not match any of the specified commands, return an error
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the Anilist server command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
/// It also checks if the Anilist module is activated for the guild. If not, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn anilist_server(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    // Define the error for when the Anilist module is off
    let anilist_module_error: AppError = AppError {
        message: String::from("Anilist module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anilist module is on for the guild
    if !check_if_module_is_on(guild_id, "ANIME").await? {
        return Err(anilist_module_error);
    }
    // Match the command name to the appropriate function
    match command_name {
        "list_user" => list_register_user::run(ctx, command_interaction).await,
        "list_activity" => list_all_activity::run(ctx, command_interaction).await,
        // If the command name does not match any of the specified commands, return an error
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the Anilist user command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
/// It also checks if the Anilist module is activated for the guild. If not, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn anilist_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    // Define the error for when the Anilist module is off
    let anilist_module_error: AppError = AppError {
        message: String::from("Anilist module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anilist module is on for the guild
    if !check_if_module_is_on(guild_id, "ANIME").await? {
        return Err(anilist_module_error);
    }
    // Match the command name to the appropriate function
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
        // If the command name does not match any of the specified commands, return an error
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the Anime command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
/// It also checks if the Anime module is activated for the guild. If not, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn anime(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    // Define the error for when the Anime module is off
    let anime_module_error: AppError = AppError {
        message: String::from("Anime module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anime module is on for the guild
    if !check_if_module_is_on(guild_id, "ANIME").await? {
        return Err(anime_module_error);
    }
    // Match the command name to the appropriate function
    match command_name {
        "random_image" => random_image::run(ctx, command_interaction).await,
        // If the command name does not match any of the specified commands, return an error
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the Anime NSFW command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
/// It also checks if the Anime NSFW module is activated for the guild. If not, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn anime_nsfw(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    // Define the error for when the Anime NSFW module is off
    let anime_module_error: AppError = AppError {
        message: String::from("Anime module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anime NSFW module is on for the guild
    if !check_if_module_is_on(guild_id, "ANIME").await? {
        return Err(anime_module_error);
    }
    // Match the command name to the appropriate function
    match command_name {
        "random_nsfw_image" => random_nsfw_image::run(ctx, command_interaction).await,
        // If the command name does not match any of the specified commands, return an error
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the Bot command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn bot(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    // Match the command name to the appropriate function
    match command_name {
        "credit" => credit::run(ctx, command_interaction).await,
        "info" => info::run(ctx, command_interaction).await,
        "ping" => ping::run(ctx, command_interaction).await,
        // If the command name does not match any of the specified commands, return an error
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the server command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn server(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "guild" => guild::run(ctx, command_interaction).await,
        "guild_image" => generate_image_pfp_server::run(ctx, command_interaction).await,
        "guild_image_g" => generate_image_pfp_server_global::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the steam command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
/// It also checks if the Game module is activated for the guild. If not, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn steam(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    let game_module_error: AppError = AppError {
        message: String::from("Game module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    if !check_if_module_is_on(guild_id, "GAME").await? {
        return Err(game_module_error);
    }
    match command_name {
        "game" => steam_game_info::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

/// Executes the user command.
///
/// This function retrieves the subcommand from the command interaction and matches it to the appropriate function.
/// If the subcommand does not match any of the specified subcommands, it returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `command_name` - The name of the command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command was dispatched successfully, or `Err` if an error occurred.
async fn user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    match command_name {
        "avatar" => avatar::run(ctx, command_interaction).await,
        "banner" => banner::run(ctx, command_interaction).await,
        "profile" => profile::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

async fn vn(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
) -> Result<(), AppError> {
    let vn_module_error: AppError = AppError {
        message: String::from("Visual novel module is off."),
        error_type: ErrorType::Module,
        error_response_type: ErrorResponseType::Message,
    };
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    if !check_if_module_is_on(guild_id, "VN").await? {
        return Err(vn_module_error);
    }
    match command_name {
        "game" => avatar::run(ctx, command_interaction).await,
        "character" => banner::run(ctx, command_interaction).await,
        "staff" => profile::run(ctx, command_interaction).await,
        "user" => profile::run(ctx, command_interaction).await,
        "producer" => profile::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Command does not exist."),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}
