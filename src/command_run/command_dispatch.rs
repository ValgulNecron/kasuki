use serenity::all::{CommandInteraction, Context};

use crate::command_run::ai::{image, question, transcript, translation};
use crate::command_run::anilist::{
    add_activity, anime, character, compare, delete_activity, level, list_all_activity,
    list_register_user, ln, manga, random, random_image, random_nsfw_image, register, search,
    seiyuu, staff, studio, user, waifu,
};
use crate::command_run::game::steam_game_info;
use crate::command_run::general::module::check_activation_status;
use crate::command_run::general::{
    avatar, banner, credit, generate_image_pfp_server, generate_image_pfp_server_global, guild,
    info, lang, module, ping, profile,
};
use crate::database::dispatcher::data_dispatch::get_data_module_activation_kill_switch_status;
use crate::database_struct::module_status::ActivationStatusModule;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn command_dispatching(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let ai_module_error = AppError::new(
        String::from("AI module is off."),
        ErrorType::Module,
        ErrorResponseType::Message,
    );

    let anilist_module_error = AppError::new(
        String::from("Anilist module is off."),
        ErrorType::Module,
        ErrorResponseType::Message,
    );

    let game_module_error = AppError::new(
        String::from("Game module is off."),
        ErrorType::Module,
        ErrorResponseType::Message,
    );
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => "0".to_string(),
    };
    match command_interaction.data.name.as_str() {
        /*

        THIS IS THE GENERAL MODULE.

         */
        "avatar" => avatar::run(ctx, command_interaction).await?,
        "banner" => banner::run(ctx, command_interaction).await?,
        "credit" => credit::run(ctx, command_interaction).await?,
        "info" => info::run(ctx, command_interaction).await?,
        "lang" => lang::run(&command_interaction.data.options, ctx, command_interaction).await?,
        "module" => module::run(ctx, command_interaction).await?,
        "ping" => ping::run(ctx, command_interaction).await?,
        "profile" => profile::run(ctx, command_interaction).await?,
        "guild" => guild::run(ctx, command_interaction).await?,
        "guild_image" => generate_image_pfp_server::run(ctx, command_interaction).await?,
        "list_activity" => list_all_activity::run(ctx, command_interaction).await?,
        "guild_image_g" => generate_image_pfp_server_global::run(ctx, command_interaction).await?,

        /*

        THIS IS THE AI MODULE.

         */
        "image" => {
            if check_if_module_is_on(guild_id, "AI").await? {
                image::run(ctx, command_interaction).await?
            } else {
                return Err(ai_module_error);
            }
        }
        "transcript" => {
            if check_if_module_is_on(guild_id, "AI").await? {
                transcript::run(ctx, command_interaction).await?
            } else {
                return Err(ai_module_error);
            }
        }
        "translation" => {
            if check_if_module_is_on(guild_id, "AI").await? {
                translation::run(ctx, command_interaction).await?
            } else {
                return Err(ai_module_error);
            }
        }
        "question" => {
            if check_if_module_is_on(guild_id, "AI").await? {
                question::run(ctx, command_interaction).await?
            } else {
                return Err(ai_module_error);
            }
        }

        /*

        THIS IS THE ANILIST MODULE.

         */
        "anime" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                anime::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "ln" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                ln::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "manga" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                manga::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "add_anime_activity" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                add_activity::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "user" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                user::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "character" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                character::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "waifu" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                waifu::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "compare" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                compare::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                random::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "register" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                register::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "staff" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                staff::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "studio" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                studio::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "search" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                search::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "seiyuu" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                seiyuu::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "level" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                level::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "list_user" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                list_register_user::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random_image" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                random_image::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random_nsfw_image" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                random_nsfw_image::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "delete_activity" => {
            if check_if_module_is_on(guild_id, "ANILIST").await? {
                delete_activity::run(ctx, command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }

        /*

        THIS IS THE GAME MODULE.

         */
        "steam_game" => {
            if check_if_module_is_on(guild_id, "GAME").await? {
                steam_game_info::run(ctx, command_interaction).await?
            } else {
                return Err(game_module_error);
            }
        }
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
    let state = check_activation_status(module, guild_id.clone()).await?;
    let state = state && check_kill_switch_status(module).await?;
    Ok(state)
}

async fn check_kill_switch_status(module: &str) -> Result<bool, AppError> {
    let row: ActivationStatusModule = get_data_module_activation_kill_switch_status().await?;
    Ok(match module {
        "ANILIST" => row.anilist_module.unwrap_or(true),
        "AI" => row.ai_module.unwrap_or(true),
        "GAME" => row.game_module.unwrap_or(true),
        "NEW_MEMBER" => row.new_member.unwrap_or(true),
        _ => false,
    })
}
