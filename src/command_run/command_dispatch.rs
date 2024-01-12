use serenity::all::{CommandInteraction, Context};

use crate::command_run::ai::{image, transcript, translation};
use crate::command_run::anilist::{
    add_activity, anime, character, compare, level, list_all_activity, list_register_user, ln,
    manga, random, random_image, random_nsfw_image, register, search, seiyuu, staff, studio, user,
    waifu,
};
use crate::command_run::game::steam_game_info;
use crate::command_run::general::module::check_activation_status;
use crate::command_run::general::{
    avatar, banner, credit, generate_image_pfp_server, guild, info, lang, module, ping, profile,
};
use crate::error_enum::AppError;
use crate::error_enum::AppError::UnknownCommandError;
use crate::sqls::general::data::get_data_module_activation_kill_switch_status;

pub async fn command_dispatching(
    ctx: Context,
    command_interaction: CommandInteraction,
) -> Result<(), AppError> {
    let ai_module_error = AppError::ModuleOffError(String::from("AI module is off."));
    let anilist_module_error = AppError::ModuleOffError(String::from("Anilist module is off."));
    match command_interaction.data.name.as_str() {
        /*

        THIS IS THE GENERAL MODULE.

         */
        "avatar" => {
            avatar::run(
                &command_interaction.data.options,
                &ctx,
                &command_interaction,
            )
            .await?
        }
        "banner" => {
            banner::run(
                &command_interaction.data.options,
                &ctx,
                &command_interaction,
            )
            .await?
        }
        "credit" => credit::run(&ctx, &command_interaction).await?,
        "info" => info::run(&ctx, &command_interaction).await?,
        "lang" => {
            lang::run(
                &command_interaction.data.options,
                &ctx,
                &command_interaction,
            )
            .await?
        }
        "module" => {
            module::run(
                &command_interaction.data.options,
                &ctx,
                &command_interaction,
            )
            .await?
        }
        "ping" => ping::run(&ctx, &command_interaction).await?,
        "profile" => {
            profile::run(
                &command_interaction.data.options,
                &ctx,
                &command_interaction,
            )
            .await?
        }
        "guild" => guild::run(&ctx, &command_interaction).await?,
        "guild_image" => generate_image_pfp_server::run(&ctx, &command_interaction).await?,
        "list_activity" => list_all_activity::run(&ctx, &command_interaction).await?,

        /*

        THIS IS THE AI MODULE.

         */
        "image" => {
            if check_if_moule_is_on(&command_interaction, "AI").await? {
                image::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(ai_module_error);
            }
        }
        "transcript" => {
            if check_if_moule_is_on(&command_interaction, "AI").await? {
                transcript::run(
                    &command_interaction.data.options(),
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(ai_module_error);
            }
        }
        "translation" => {
            if check_if_moule_is_on(&command_interaction, "AI").await? {
                translation::run(
                    &command_interaction.data.options(),
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(ai_module_error);
            }
        }

        /*

        THIS IS THE ANILIST MODULE.

         */
        "anime" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                anime::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "ln" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                ln::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "manga" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                manga::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "add_activity" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                add_activity::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "user" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                user::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "character" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                character::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "waifu" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                waifu::run(&ctx, &command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "compare" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                compare::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                random::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "register" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                register::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "staff" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                staff::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "studio" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                studio::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "search" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                search::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "seiyuu" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                seiyuu::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "level" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                level::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "list_user" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                list_register_user::run(&ctx, &command_interaction).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random_image" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                random_image::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random_nsfw_image" => {
            if check_if_moule_is_on(&command_interaction, "ANILIST").await? {
                random_nsfw_image::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }

        /*

        THIS IS THE GAME MODULE.

         */
        "steam_game" => {
            if check_if_moule_is_on(&command_interaction, "GAME").await? {
                steam_game_info::run(
                    &command_interaction.data.options,
                    &ctx,
                    &command_interaction,
                )
                .await?
            } else {
                return Err(anilist_module_error);
            }
        }
        _ => return Err(UnknownCommandError(String::from("Command does not exist."))),
    }

    Ok(())
}

async fn check_if_moule_is_on(
    command_interaction: &CommandInteraction,
    module: &str,
) -> Result<bool, AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => return Ok(true),
    };
    let state = check_activation_status(module, guild_id.clone()).await?;
    let state = state && check_kill_switch_status(module).await?;
    Ok(state)
}

async fn check_kill_switch_status(module: &str) -> Result<bool, AppError> {
    let row: (Option<String>, Option<bool>, Option<bool>, Option<bool>) =
        get_data_module_activation_kill_switch_status().await?;
    let (_, ai_module, anilist_module, game_module): (
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
    ) = row;
    Ok(match module {
        "ANILIST" => anilist_module.unwrap_or(true),
        "AI" => ai_module.unwrap_or(true),
        "GAME" => game_module.unwrap_or(true),
        _ => false,
    })
}
