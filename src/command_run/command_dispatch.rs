use crate::command_run::general::credit;
use crate::error_enum::AppError;
use crate::error_enum::AppError::UnknownCommandError;
use serenity::all::{CommandInteraction, Context};

pub async fn command_dispatching(
    ctx: Context,
    command: CommandInteraction,
) -> Result<(), AppError> {
    let ai_module_error = AppError::ModuleOffError(String::from("AI module is off."));
    let anilist_module_error = AppError::ModuleOffError(String::from("Anilist module is off."));
    match command.data.name.as_str() {
        "credit" => credit::run(&ctx, &command).await?,
        _ => return Err(UnknownCommandError(String::from("Command does not exist."))),
    }

    Ok(())
}
