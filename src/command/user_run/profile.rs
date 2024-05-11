use serenity::all::{CommandInteraction, Context, User};

use crate::command::run::user::profile::profile_with_user;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// This function is responsible for running the profile command.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context, which holds data about the current state of the application.
/// * `command_interaction` - A reference to the CommandInteraction, which holds data about the command that was invoked.
///
/// # Returns
///
/// This function returns a Result. If the function executes successfully, it returns Ok(()). If there is an error, it returns Err(AppError).
///
/// # Asynchronous
///
/// This function is asynchronous, meaning it will return immediately, and the actual work will be done in the background.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Get a reference to the users involved in the command interaction
    let users = &command_interaction.data.resolved.users;

    // Initialize a mutable reference to a default user
    let mut user: Option<User> = None;
    let command_user = command_interaction.user.clone();

    // Iterate over the users
    for (user_id, u) in users {
        // If the user_id is not the same as the id of the user who invoked the command, assign the user to u and break the loop
        if user_id != &command_interaction.user.id {
            user = Some(u.clone());
            break;
        }
    }

    let user = user.unwrap_or(command_user);
    let user = user.id.to_user(&ctx.http).await.map_err(|e| {
        AppError::new(
            format!("Could not get the user. {}", e),
            ErrorType::Option,
            ErrorResponseType::Message,
        )
    })?;

    // Call the profile_with_user function with the context, command interaction, and user
    profile_with_user(ctx, command_interaction, &user).await
}
