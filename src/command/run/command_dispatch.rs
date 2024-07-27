use std::error::Error;

use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateInteractionResponseMessage, EntitlementKind,
    SkuFlags, SkuId, SkuKind,
};
use tracing::trace;

use crate::command::run::admin::anilist::{add_activity, delete_activity};
use crate::command::run::admin::server::module;
use crate::command::run::admin::server::module::check_activation_status;
use crate::command::run::admin::server::{lang, new_member_setting};
use crate::command::run::ai::{image, question, transcript, translation};
use crate::command::run::anilist_server::{list_all_activity, list_register_user};
use crate::command::run::anilist_user::{
    anime, character, compare, level, ln, manga, random, register, search, seiyuu, staff, studio,
    user, waifu,
};
use crate::command::run::anime::random_image;
use crate::command::run::anime_nsfw::random_nsfw_image;
use crate::command::run::bot::{credit, info, ping};
use crate::command::run::management::{give_premium_sub, kill_switch, remove_test_sub};
use crate::command::run::server::{
    generate_image_pfp_server, generate_image_pfp_server_global, guild,
};
use crate::command::run::steam::steam_game_info;
use crate::command::run::user::{avatar, banner, command_usage, profile};
use crate::command::run::vn;
use crate::command::run::vn::{game, producer, stats};
use crate::constant::{MAX_FREE_AI_IMAGES, PAID_MULTIPLIER};
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::manage::dispatcher::data_dispatch::{
    get_data_module_activation_kill_switch_status, get_data_module_activation_status,
};
use crate::event_handler::Handler;
use crate::helper::error_management::error_enum::ResponseError;
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
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the command name from the command interaction
    let command_name = command_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str();
    let full_command_name = command_interaction.data.name.as_str();
    let full_command_name = format!("{} {}", full_command_name, command_name);
    // Match the command name to the appropriate function
    match command_interaction.data.name.as_str() {
        // anilist_user module
        "admin" => {
            admin(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "ai" => {
            ai(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "anilist_server" => {
            anilist_server(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "anilist_user" => {
            anilist_user(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "anime" => {
            anime(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "hanime" => {
            anime_nsfw(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "bot" => {
            bot(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "server" => {
            server(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "steam" => {
            steam(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "user" => {
            user(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "vn" => {
            vn(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        // Management
        "kill_switch" => {
            kill_switch::run(
                ctx,
                command_interaction,
                self_handler.bot_data.config.clone(),
            )
            .await?
        }
        "give_premium_sub" => {
            give_premium_sub::run(
                ctx,
                command_interaction,
                self_handler.bot_data.config.clone(),
            )
            .await?
        }
        "remove_test_sub" => {
            remove_test_sub::run(
                ctx,
                command_interaction,                self_handler.bot_data.config.clone(),

            )
            .await?
        }

        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))));
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
pub async fn check_if_module_is_on(
    guild_id: String,
    module: &str,
    db_type: String,
) -> Result<bool, Box<dyn Error>> {
    let row: ActivationStatusModule =
        get_data_module_activation_status(&guild_id, db_type.clone()).await?;
    trace!(?row);
    let state = check_activation_status(module, row).await;
    trace!(state);
    let state = state && check_kill_switch_status(module, db_type).await?;
    trace!(state);
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
async fn check_kill_switch_status(module: &str, db_type: String) -> Result<bool, Box<dyn Error>> {
    let row: ActivationStatusModule =
        get_data_module_activation_kill_switch_status(db_type).await?;
    trace!(?row);
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone();
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    let anime_module_error = ResponseError::Option(String::from(
        "Anime module is not enabled. Please enable it first.",
    ));
    trace!(command_name);
    let subcommand = get_subcommand(command_interaction).unwrap();
    let subcommand_name = subcommand.name;
    let full_command_name = format!("{} {}", full_command_name, subcommand_name);

    match command_name {
        "general" => {
            general_admin(
                ctx,
                command_interaction,
                subcommand_name,
                full_command_name,
                self_handler,
            )
            .await
        }
        "anilist" => {
            if check_if_module_is_on(guild_id, "ANIME", db_type).await? {
                anilist_admin(
                    ctx,
                    command_interaction,
                    subcommand_name,
                    full_command_name,
                    self_handler,
                )
                .await
            } else {
                Err(Box::new(anime_module_error))
            }
        }
        _ => Err(Box::new(ResponseError::Option(String::from(
            "Unknown command",
        )))),
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let anilist_cache = self_handler.bot_data.anilist_cache.clone();
    let return_data = match command_name {
        "add_anime_activity" => {
            add_activity::run(ctx, command_interaction, config, anilist_cache).await
        }
        "delete_activity" => {
            delete_activity::run(ctx, command_interaction, config, anilist_cache).await
        }
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let return_data = match command_name {
        "lang" => lang::run(ctx, command_interaction, config).await,
        "module" => module::run(ctx, command_interaction, config).await,
        "member_info_setting" => new_member_setting::run(ctx, command_interaction, config).await,
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone();
    // Define the error for when the AI module is off
    let ai_module_error = ResponseError::Option(String::from(
        "AI module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the AI module is on for the guild
    if !check_if_module_is_on(guild_id, "AI", db_type).await? {
        return Err(Box::new(ai_module_error));
    }
    // Match the command name to the appropriate function
    let return_data = match command_name {
        "image" => {
            let limit =
                check_hourly_limit(command_interaction, ctx, &full_command_name, self_handler)
                    .await?;
            if limit {
                return Ok(());
            } else {
                image::run(ctx, command_interaction, config).await
            }
        }
        "transcript" => transcript::run(ctx, command_interaction, config).await,
        "translation" => translation::run(ctx, command_interaction, config).await,
        "question" => question::run(ctx, command_interaction, config).await,
        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
}

async fn check_hourly_limit(
    command_interaction: &CommandInteraction,
    ctx: &Context,
    full_command_name: &String,
    self_handler: &Handler,
) -> Result<bool, Box<dyn Error>> {
    // get the first sku that is a user subscription
    let user_skus: Vec<SkuId> = command_interaction
        .entitlements
        .iter()
        .map(|entitlement| entitlement.sku_id)
        .collect();
    let available_skus = ctx.http.get_skus().await.map_err(|e| {
        ResponseError::Sending(format!("Error while sending the premium: {:#?}", e))
    })?;
    let mut user_sub = None;
    for available_sku in available_skus {
        match available_sku.kind {
            SkuKind::Subscription => match available_sku.flags {
                SkuFlags::USER_SUBSCRIPTION => {
                    if user_sub.is_none() {
                        if user_skus.contains(&available_sku.id) {
                            user_sub = Some(available_sku.id);
                        }
                    }
                }
                _ => {}
            },
            SkuKind::SubscriptionGroup => {}
            SkuKind::Unknown(_) => {}
            _ => {}
        }
    }
    let usage = self_handler
        .get_hourly_usage(
            full_command_name.clone(),
            command_interaction.user.id.to_string(),
        )
        .await;
    if usage <= MAX_FREE_AI_IMAGES as u128 && user_sub.is_none() {
        return Ok(false);
    }

    if usage <= (MAX_FREE_AI_IMAGES as f64 * PAID_MULTIPLIER) as u128 && user_sub.is_some() {
        return Ok(false);
    }

    let premium_button = CreateButton::new_premium(user_sub.unwrap());
    let builder = CreateInteractionResponseMessage::new();
    let builder = builder.button(premium_button);
    let builder = serenity::builder::CreateInteractionResponse::Message(builder);
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

    Ok(true)
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone(); // Define the error for when the Anilist module is off
    let anilist_cache = self_handler.bot_data.anilist_cache.clone();
    let anilist_module_error = ResponseError::Option(String::from(
        "Anilist module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anilist module is on for the guild
    if !check_if_module_is_on(guild_id, "ANILIST", db_type).await? {
        return Err(Box::new(anilist_module_error));
    }
    // Match the command name to the appropriate function
    let return_data = match command_name {
        "list_user" => {
            list_register_user::run(ctx, command_interaction, config, anilist_cache).await
        }
        "list_activity" => list_all_activity::run(ctx, command_interaction, config).await,
        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone(); // Define the error for when the Anilist module is off
    let anilist_cache = self_handler.bot_data.anilist_cache.clone();
    let anilist_module_error = ResponseError::Option(String::from(
        "Anilist module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anilist module is on for the guild
    if !check_if_module_is_on(guild_id, "ANILIST", db_type).await? {
        return Err(Box::new(anilist_module_error));
    }
    // Match the command name to the appropriate function
    let return_data = match command_name {
        "anime" => anime::run(ctx, command_interaction, config, anilist_cache).await,
        "ln" => ln::run(ctx, command_interaction, config, anilist_cache).await,
        "manga" => manga::run(ctx, command_interaction, config, anilist_cache).await,
        "user" => user::run(ctx, command_interaction, config, anilist_cache).await,
        "character" => character::run(ctx, command_interaction, config, anilist_cache).await,
        "waifu" => waifu::run(ctx, command_interaction, config, anilist_cache).await,
        "compare" => compare::run(ctx, command_interaction, config, anilist_cache).await,
        "random" => random::run(ctx, command_interaction, config, anilist_cache).await,
        "register" => register::run(ctx, command_interaction, config, anilist_cache).await,
        "staff" => staff::run(ctx, command_interaction, config, anilist_cache).await,
        "studio" => studio::run(ctx, command_interaction, config, anilist_cache).await,
        "search" => search::run(ctx, command_interaction, config, anilist_cache).await,
        "seiyuu" => seiyuu::run(ctx, command_interaction, config, anilist_cache).await,
        "level" => level::run(ctx, command_interaction, config, anilist_cache).await,
        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone(); // Define the error for when the Anime module is off
    let anime_module_error = ResponseError::Option(String::from(
        "Anime module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anime module is on for the guild
    if !check_if_module_is_on(guild_id, "ANIME", db_type).await? {
        return Err(Box::new(anime_module_error));
    }
    // Match the command name to the appropriate function
    let return_data = match command_name {
        "random_image" => random_image::run(ctx, command_interaction, config).await,
        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone(); // Define the error for when the Anime NSFW module is off
    let anime_module_error = ResponseError::Option(String::from(
        "Anime module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anime NSFW module is on for the guild
    if !check_if_module_is_on(guild_id, "ANIME", db_type).await? {
        return Err(Box::new(anime_module_error));
    }
    // Match the command name to the appropriate function
    let return_data = match command_name {
        "random_himage" => random_nsfw_image::run(ctx, command_interaction, config).await,
        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    // Match the command name to the appropriate function
    let return_data = match command_name {
        "credit" => credit::run(ctx, command_interaction, config).await,
        "info" => info::run(ctx, command_interaction, config).await,
        "ping" => ping::run(ctx, command_interaction, config).await,
        // If the command name does not match any of the specified commands, return an error
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let return_data = match command_name {
        "guild" => guild::run(ctx, command_interaction, config).await,
        "guild_image" => generate_image_pfp_server::run(ctx, command_interaction, config).await,
        "guild_image_g" => {
            generate_image_pfp_server_global::run(ctx, command_interaction, config).await
        }
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;
    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone();
    let apps = self_handler.bot_data.apps.clone();
    let game_module_error = ResponseError::Option(String::from(
        "The Game module is not activated for this guild.",
    ));
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    if !check_if_module_is_on(guild_id, "GAME", db_type).await? {
        return Err(Box::new(game_module_error));
    }
    let return_data = match command_name {
        "game" => steam_game_info::run(ctx, command_interaction, config, apps).await,
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;
    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
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
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let return_data = match command_name {
        "avatar" => avatar::run(ctx, command_interaction, config).await,
        "banner" => banner::run(ctx, command_interaction, config).await,
        "profile" => profile::run(ctx, command_interaction, config).await,
        "command_usage" => command_usage::run(ctx, command_interaction, self_handler).await,
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;
    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
}

async fn vn(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone();
    let vndb_cache = self_handler.bot_data.vndb_cache.clone();
    let vn_module_error = ResponseError::Option(String::from(
        "The VN module is not activated for this guild.",
    ));
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    if !check_if_module_is_on(guild_id, "VN", db_type).await? {
        return Err(Box::new(vn_module_error));
    }
    let return_data = match command_name {
        "game" => game::run(ctx, command_interaction, config, vndb_cache).await,
        "character" => vn::character::run(ctx, command_interaction, config, vndb_cache).await,
        "staff" => vn::staff::run(ctx, command_interaction, config, vndb_cache).await,
        "user" => vn::user::run(ctx, command_interaction, config, vndb_cache).await,
        "producer" => producer::run(ctx, command_interaction, config, vndb_cache).await,
        "stats" => stats::run(ctx, command_interaction, config, vndb_cache).await,
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Unknown command",
            ))))
        }
    };
    return_data?;
    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;
    Ok(())
}
