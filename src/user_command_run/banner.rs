use serenity::all::{CommandInteraction, Context};
use crate::command_run::user::avatar::avatar_with_user;
use crate::command_run::user::banner::banner_with_user;
use crate::error_management::error_enum::AppError;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let users = &command_interaction.data.resolved.users;
    let mut user = &Default::default();
    for (user_id, u) in users {
        if user_id != &command_interaction.user.id {
            user = u;
            break;
        }
    }
    banner_with_user(ctx, command_interaction, user).await
}
