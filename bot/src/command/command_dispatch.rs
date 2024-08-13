use crate::command::admin::anilist::add_activity::AddActivityCommand;
use crate::command::admin::anilist::delete_activity::DeleteActivityCommand;
use crate::command::admin::server::lang::LangCommand;
use crate::command::admin::server::module::{check_activation_status, ModuleCommand};
use crate::command::admin::server::new_member_setting::NewMemberSettingCommand;
use crate::command::ai::image::ImageCommand;
use crate::command::ai::question::QuestionCommand;
use crate::command::ai::transcript::TranscriptCommand;
use crate::command::ai::translation::TranslationCommand;
use crate::command::anilist_server::list_all_activity::ListAllActivity;
use crate::command::anilist_server::list_register_user::ListRegisterUser;
use crate::command::anilist_user::anime::AnimeCommand;
use crate::command::anilist_user::character::CharacterCommand;
use crate::command::anilist_user::compare::CompareCommand;
use crate::command::anilist_user::level::LevelCommand;
use crate::command::anilist_user::ln::LnCommand;
use crate::command::anilist_user::manga::MangaCommand;
use crate::command::anilist_user::random::RandomCommand;
use crate::command::anilist_user::register::RegisterCommand;
use crate::command::anilist_user::search::SearchCommand;
use crate::command::anilist_user::seiyuu::SeiyuuCommand;
use crate::command::anilist_user::staff::StaffCommand;
use crate::command::anilist_user::studio::StudioCommand;
use crate::command::anilist_user::user::UserCommand;
use crate::command::anilist_user::waifu::WaifuCommand;
use crate::command::command_trait::SlashCommand;
use crate::command::guess_kind::guess_command_kind;
use crate::command::run::anime::random_image;
use crate::command::run::anime_nsfw::random_nsfw_image;
use crate::command::run::audio::join;
use crate::command::run::audio::play;
use crate::command::run::bot::{credit, info, ping};
use crate::command::run::management::{give_premium_sub, kill_switch, remove_test_sub};
use crate::command::run::server::{
    generate_image_pfp_server, generate_image_pfp_server_global, guild,
};
use crate::command::run::vn;
use crate::command::run::vn::{game, producer, stats};
use crate::command::steam::steam_game_info::SteamGameInfoCommand;
use crate::command::user::avatar::AvatarCommand;
use crate::command::user::banner::BannerCommand;
use crate::command::user::command_usage::CommandUsageCommand;
use crate::command::user::profile::ProfileCommand;
use crate::config::BotConfigDetails;
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::manage::dispatcher::data_dispatch::{
    get_data_module_activation_kill_switch_status, get_data_module_activation_status,
};
use crate::event_handler::Handler;
use crate::helper::error_management::error_enum::ResponseError;
use serenity::all::{
    CommandInteraction, Context,
};
use std::error::Error;
use tracing::trace;

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
pub async fn dispatch_command(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let (kind, name) = guess_command_kind(command_interaction);
    let full_command_name = format!("{} {}", kind, name);
    match name.as_str() {
        "user_avatar" => {
            return AvatarCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }
        "user_banner" => {
            return BannerCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }
        "user_profile" => {
            return ProfileCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }
        "user_command_usage" => {
            return CommandUsageCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                command_usage: self_handler
                    .bot_data
                    .number_of_command_use_per_command
                    .clone(),
            }
            .run_slash()
            .await;
        }

        "admin_general_lang" => {
            return LangCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }
        "admin_general_module" => {
            return ModuleCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }
        "admin_general_new_member_setting" => {
            return NewMemberSettingCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }

        "admin_anilist_add_activity" => {
            return AddActivityCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "admin_anilist_delete_activity" => {
            return DeleteActivityCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }

        "steam_game" => {
            return SteamGameInfoCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                apps: self_handler.bot_data.apps.clone(),
            }
            .run_slash()
            .await;
        }

        "ai_image" => {
            return ImageCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await;
        }
        "ai_question" => {
            return QuestionCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await;
        }
        "ai_transcript" => {
            return TranscriptCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await;
        }
        "ai_translation" => {
            return TranslationCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await;
        }

        "list_user" => {
            return ListRegisterUser {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "list_activity" => {
            return ListAllActivity {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await;
        }

        "anime" => {
            return AnimeCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "character" => {
            return CharacterCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "compare" => {
            return CompareCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "level" => {
            return LevelCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "ln" => {
            return LnCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "manga" => {
            return MangaCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "anilist_user" => {
            return UserCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "waifu" => {
            return WaifuCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "random" => {
            return RandomCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "register" => {
            return RegisterCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "staff" => {
            return StaffCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "studio" => {
            return StudioCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "search" => {
            return SearchCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        "seiyuu" => {
            return SeiyuuCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await;
        }
        _ => {}
    }
    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;

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
    let config = self_handler.bot_data.config.clone();
    let anilist_cache = self_handler.bot_data.anilist_cache.clone();
    // Match the command name to the appropriate function
    match command_interaction.data.name.as_str() {
        // anilist module user command
        "random_anime" => {
            anime(
                ctx,
                command_interaction,
                command_name,
                full_command_name,
                self_handler,
            )
            .await?
        }
        "random_hanime" => {
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

        "audio" => {
            audio(
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
                command_interaction,
                self_handler.bot_data.config.clone(),
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
    db_config: BotConfigDetails,
) -> Result<bool, Box<dyn Error>> {
    let row: ActivationStatusModule =
        get_data_module_activation_status(&guild_id, db_type.clone(), db_config.clone()).await?;
    trace!(?row);
    let state = check_activation_status(module, row).await;
    trace!(state);
    let state = state && check_kill_switch_status(module, db_type, db_config).await?;
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
async fn check_kill_switch_status(
    module: &str,
    db_type: String,
    db_config: BotConfigDetails,
) -> Result<bool, Box<dyn Error>> {
    let row: ActivationStatusModule =
        get_data_module_activation_kill_switch_status(db_type, db_config).await?;
    trace!(?row);
    Ok(check_activation_status(module, row).await)
}

async fn audio(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_name: &str,
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone();
    // Define the error for when the AI module is off
    let audio_module_error = ResponseError::Option(String::from(
        "Audio module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    let return_data = match command_name {
        "play" => play::run(ctx, command_interaction, config).await,
        "join" => join::run(ctx, command_interaction, config).await,
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
    command_interaction: &CommandInteraction,
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone(); // Define the error for when the Anilist module is off
    let anilist_module_error = ResponseError::Option(String::from(
        "Anilist module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anilist module is on for the guild
    if !check_if_module_is_on(guild_id, "ANILIST", db_type, config.bot.config.clone()).await? {
        return Err(Box::new(anilist_module_error));
    }

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
    command_interaction: &CommandInteraction,
    full_command_name: String,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let config = self_handler.bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone(); // Define the error for when the Anilist module is off
    let anilist_module_error = ResponseError::Option(String::from(
        "Anilist module is not activated. Please enable it first.",
    ));
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    // Check if the Anilist module is on for the guild
    if !check_if_module_is_on(guild_id, "ANILIST", db_type, config.bot.config.clone()).await? {
        return Err(Box::new(anilist_module_error));
    }

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
    if !check_if_module_is_on(guild_id, "ANIME", db_type, config.bot.config.clone()).await? {
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
    if !check_if_module_is_on(guild_id, "ANIME", db_type, config.bot.config.clone()).await? {
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
    if !check_if_module_is_on(guild_id, "VN", db_type, config.bot.config.clone()).await? {
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
