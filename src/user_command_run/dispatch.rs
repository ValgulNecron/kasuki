use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::user_command_run::avatar::send_user_avatar;
use serenity::all::{CommandInteraction, Context};

pub async fn dispatch_user_command(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    match command_interaction.data.name.as_str() {
        "avatar" => send_user_avatar(ctx, command_interaction).await,
        _ => {
            return Err(AppError::new(
                String::from("Command does not exist."),
                ErrorType::Option,
                ErrorResponseType::Message,
            ))
        }
    }
}
