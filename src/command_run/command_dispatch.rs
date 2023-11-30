use crate::command_run::general::{avatar, banner, credit, info, lang, module};
use crate::error_enum::AppError;
use crate::error_enum::AppError::UnknownCommandError;
use log::info;
use serenity::all::{CommandInteraction, Context};

pub async fn command_dispatching(
    ctx: Context,
    command: CommandInteraction,
) -> Result<(), AppError> {
    info!("{:?}", command);
    let ai_module_error = AppError::ModuleOffError(String::from("AI module is off."));
    let anilist_module_error = AppError::ModuleOffError(String::from("Anilist module is off."));
    match command.data.name.as_str() {
        "avatar" => avatar::run(&command.data.options, &ctx, &command).await?,
        "banner" => banner::run(&command.data.options, &ctx, &command).await?,
        "credit" => credit::run(&ctx, &command).await?,
        "info" => info::run(&ctx, &command).await?,
        "lang" => lang::run(&command.data.options, &ctx, &command).await?,
        "module" => module::run(&command.data.options, &ctx, &command).await?,
        _ => return Err(UnknownCommandError(String::from("Command does not exist."))),
    }

    Ok(())
}
