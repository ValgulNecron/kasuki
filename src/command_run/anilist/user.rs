use serenity::all::{CommandDataOption, CommandInteraction, Context};
use tracing::trace;

use crate::anilist_struct::run::user::{send_embed, UserWrapper};
use crate::database::dispatcher::data_dispatch::get_registered_user;
use crate::error_management::error_enum::AppError;
use crate::error_management::error_enum::AppError::Error;
use crate::error_management::error_enum::CommandError::ErrorOptionError;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    trace!("{:?}", options);
    for option in options {
        if option.name.as_str() != "type" {
            if let Some(a) = option.value.as_str() {
                let value = &a.to_string();

                let data: UserWrapper = get_user_data(value).await?;

                return send_embed(ctx, command_interaction, data).await;
            }
        }
    }
    let user_id = &command_interaction.user.id.to_string();
    let row: (Option<String>, Option<String>) = get_registered_user(user_id).await?;
    trace!("{:?}", row);
    let (user, _): (Option<String>, Option<String>) = row;
    let user = user.ok_or(Error(ErrorOptionError(String::from("There is no option"))))?;

    let data = UserWrapper::new_user_by_id(user.parse::<i32>().unwrap()).await?;
    send_embed(ctx, command_interaction, data).await
}

pub async fn get_user_data(value: &String) -> Result<UserWrapper, AppError> {
    if value.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value.parse().unwrap()).await
    } else {
        UserWrapper::new_user_by_search(value).await
    }
}
