use serenity::all::{CommandInteraction, Context};

use crate::command_run::server::generate_image_pfp_server::send_embed;
use crate::error_management::error_enum::AppError;

/// Executes the command to send an embed with the global server's profile picture.
///
/// This function calls the `send_embed` function with the `image_type` set to "global".
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    send_embed(ctx, command_interaction, "global").await
}
