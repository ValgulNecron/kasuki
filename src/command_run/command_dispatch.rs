use crate::command_run::ai::{image, transcript};
use crate::command_run::anilist::anime;
use crate::command_run::general::module::check_activation_status;
use crate::command_run::general::{avatar, banner, credit, info, lang, module, ping, profile};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{LangageGuildIdError, UnknownCommandError};
use crate::sqls::sqlite::data::get_data_module_activation_kill_switch_status_sqlite;
use serenity::all::{CommandInteraction, Context};
use tracing::info;

pub async fn command_dispatching(
    ctx: Context,
    command: CommandInteraction,
) -> Result<(), AppError> {
    info!("{:?}", command);
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

        /*

        THIS IS THE ANILIST MODULE.

         */
        "anime" => {
            if check_if_anilist_moule_is_on(&command).await? {
                anime::run(&command.data.options, &ctx, &command).await?
            } else {
                return Err(ai_module_error);
            }
        }

        _ => return Err(UnknownCommandError(String::from("Command does not exist."))),
    }

    Ok(())
}

async fn check_if_ai_moule_is_on(command: &CommandInteraction) -> Result<bool, AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();
    let state = check_activation_status("AI", guild_id.clone()).await?;
    let state = state && check_kill_switch_status("AI").await?;
    Ok(state)
}

async fn check_kill_switch_status(module: &str) -> Result<bool, AppError> {
    let row: (Option<String>, Option<bool>, Option<bool>) =
        get_data_module_activation_kill_switch_status_sqlite().await?;
    let (_, ai_module, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;
    Ok(match module {
        "ANILIST" => anilist_module.unwrap_or(true),
        "AI" => ai_module.unwrap_or(true),
        _ => false,
    })
}

async fn check_if_anilist_moule_is_on(command: &CommandInteraction) -> Result<bool, AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();
    let state = check_activation_status("ANILIST", guild_id.clone()).await?;
    let state = state && check_kill_switch_status("ANILIST").await?;
    Ok(state)
}
