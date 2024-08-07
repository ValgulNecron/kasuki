use std::error::Error;

use serenity::all::{CommandInteraction, Context};

use crate::command::user_run::{avatar, banner, profile};
use crate::event_handler::Handler;
use crate::helper::error_management::error_enum::ResponseError;

/// Dispatches the user command received from the Discord interaction.
///
/// This function takes in the context of the command and the command interaction itself.
/// It matches the name of the command to the appropriate function to handle the command.
/// Currently, it only supports the "avatar" command, which is handled by the `send_user_avatar` function.
/// If the command is not recognized, it returns an `AppError` indicating that the command does not exist.
///
/// # Arguments
///
/// * `ctx` - The context in which the command was called. This includes data like the message that triggered the command and the current bot state.
/// * `command_interaction` - The interaction data for the command. This includes data like the command name and options.
///
/// # Returns
///
/// * `Ok(())` if the command was successfully dispatched
/// * `Err(AppError)` if there was an error dispatching the command. This could be because the command does not exist or there was an error executing the command.
///
/// # Errors
///
/// This function will return an error if the command does not exist or there was an error executing the command.
pub async fn dispatch_user_command(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let db_type = self_handler.bot_data.config.bot.config.db_type.clone();
    match command_interaction.data.name.as_str() {
        "avatar" => {
            avatar::run(
                ctx,
                command_interaction,
                db_type,
                self_handler.bot_data.config.bot.config.clone(),
            )
            .await
        }
        "banner" => {
            banner::run(
                ctx,
                command_interaction,
                db_type,
                self_handler.bot_data.config.bot.config.clone(),
            )
            .await
        }
        "profile" => {
            profile::run(
                ctx,
                command_interaction,
                db_type,
                self_handler.bot_data.config.bot.config.clone(),
            )
            .await
        }
        _ => Err(Box::new(ResponseError::Option(String::from(
            "Unknown command",
        )))),
    }
}
