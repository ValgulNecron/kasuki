use crate::command_run::general::avatar::avatar_with_user;
use crate::error_management::error_enum::AppError;
use serenity::all::{CommandInteraction, Context};

pub async fn send_user_avatar(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let users = &command_interaction.data.resolved.users;
    let mut user = &Default::default();
    for (user_id, u) in users {
        if user_id != &command_interaction.user.id {
            user = u;
            break;
        }
    }
    avatar_with_user(ctx, command_interaction, user).await
}
