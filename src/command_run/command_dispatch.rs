use serenity::all::{CommandInteraction, Context};

use crate::command_run::ai::{image, transcript, translation};
use crate::command_run::anilist::{
    add_activity, anime, character, compare, level, ln, manga, random, register, search, seiyuu,
    staff, studio, user, waifu,
};
use crate::command_run::general::module::check_activation_status;
use crate::command_run::general::{avatar, banner, credit, info, lang, module, ping, profile};
use crate::error_enum::AppError;
use crate::error_enum::AppError::UnknownCommandError;
use crate::sqls::general::data::get_data_module_activation_kill_switch_status;

pub async fn command_dispatching(
    ctx: Context,
    command: CommandInteraction,
) -> Result<(), AppError> {
    let ai_module_error = AppError::ModuleOffError(String::from("AI module is off."));
    let anilist_module_error = AppError::ModuleOffError(String::from("Anilist module is off."));
    match command.data.name.as_str() {
        /*

        THIS IS THE GENERAL MODULE.

         */
        "avatar" => avatar::run(&command.data.options, &ctx, &command).await?,
        "banner" => banner::run(&command.data.options, &ctx, &command).await?,
        "credit" => credit::run(&ctx, &command).await?,
        "info" => info::run(&ctx, &command).await?,
        "lang_struct" => lang::run(&command.data.options, &ctx, &command).await?,
        "module" => module::run(&command.data.options, &ctx, &command).await?,
        "ping" => ping::run(&ctx, &command).await?,
        "profile" => profile::run(&command.data.options, &ctx, &command).await?,

        /*

        THIS IS THE AI MODULE.

         */
        "image" => {
            if check_if_ai_moule_is_on(&command).await? {
                image::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(ai_module_error);
            }
        }
        "transcript" => {
            if check_if_ai_moule_is_on(&command).await? {
                transcript::run(&command.data.options(), &ctx, &command).await?
            } else {
                return Err(ai_module_error);
            }
        }
        "translation" => {
            if check_if_ai_moule_is_on(&command).await? {
                translation::run(&command.data.options(), &ctx, &command).await?
            } else {
                return Err(ai_module_error);
            }
        }

        /*

        THIS IS THE ANILIST MODULE.

         */
        "anime" => {
            if check_if_anilist_moule_is_on(&command).await? {
                anime::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "ln" => {
            if check_if_anilist_moule_is_on(&command).await? {
                ln::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "manga" => {
            if check_if_anilist_moule_is_on(&command).await? {
                manga::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "add_activity" => {
            if check_if_anilist_moule_is_on(&command).await? {
                add_activity::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "user" => {
            if check_if_anilist_moule_is_on(&command).await? {
                user::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "character" => {
            if check_if_anilist_moule_is_on(&command).await? {
                character::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "waifu" => {
            if check_if_anilist_moule_is_on(&command).await? {
                waifu::run(&ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "compare" => {
            if check_if_anilist_moule_is_on(&command).await? {
                compare::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "random" => {
            if check_if_anilist_moule_is_on(&command).await? {
                random::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "register" => {
            if check_if_anilist_moule_is_on(&command).await? {
                register::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "staff" => {
            if check_if_anilist_moule_is_on(&command).await? {
                staff::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "studio" => {
            if check_if_anilist_moule_is_on(&command).await? {
                studio::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "search" => {
            if check_if_anilist_moule_is_on(&command).await? {
                search::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "seiyuu" => {
            if check_if_anilist_moule_is_on(&command).await? {
                seiyuu::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        "level" => {
            if check_if_anilist_moule_is_on(&command).await? {
                level::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(anilist_module_error);
            }
        }
        _ => return Err(UnknownCommandError(String::from("Command does not exist."))),
    }

    Ok(())
}

async fn check_if_ai_moule_is_on(command: &CommandInteraction) -> Result<bool, AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => return Ok(true),
    };
    let state = check_activation_status("AI", guild_id.clone()).await?;
    let state = state && check_kill_switch_status("AI").await?;
    Ok(state)
}

async fn check_kill_switch_status(module: &str) -> Result<bool, AppError> {
    let row: (Option<String>, Option<bool>, Option<bool>) =
        get_data_module_activation_kill_switch_status().await?;
    let (_, ai_module, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;
    Ok(match module {
        "ANILIST" => anilist_module.unwrap_or(true),
        "AI" => ai_module.unwrap_or(true),
        _ => false,
    })
}

async fn check_if_anilist_moule_is_on(command: &CommandInteraction) -> Result<bool, AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => return Ok(true),
    };
    let state = check_activation_status("ANILIST", guild_id.clone()).await?;
    let state = state && check_kill_switch_status("ANILIST").await?;
    Ok(state)
}
