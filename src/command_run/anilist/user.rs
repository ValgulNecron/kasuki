use serenity::all::{CommandDataOption, CommandInteraction, Context};
use tracing::trace;

use crate::anilist_struct::run::user::{send_embed, UserWrapper};
use crate::command_run::get_option::get_option_map_string;
use crate::constant::DEFAULT_STRING;
use crate::database::dispatcher::data_dispatch::get_registered_user;
use crate::error_management::command_error::CommandError;
use crate::error_management::command_error::CommandError::Generic;
use crate::error_management::generic_error::GenericError::OptionError;
use crate::error_management::interaction_error::InteractionError;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    let map = get_option_map_string(command_interaction);
    let user = map.get(&String::from("username"));

    if let Some(value) = user {
        let data: UserWrapper = get_user_data(value)
            .await
            .map_err(|e| CommandError::WebRequestError(e))?;

        return send_embed(ctx, command_interaction, data).await;
    }

    let user_id = &command_interaction.user.id.to_string();
    let row: (Option<String>, Option<String>) = get_registered_user(user_id).await?;
    trace!("{:?}", row);
    let (user, _): (Option<String>, Option<String>) = row;
    let user = user.ok_or(InteractionError::Command(Generic(OptionError(
        String::from("There is no option"),
    ))))?;

    let data = UserWrapper::new_user_by_id(user.parse::<i32>().unwrap())
        .await
        .map_err(|e| CommandError::WebRequestError(e))?;
    send_embed(ctx, command_interaction, data).await
}

pub async fn get_user_data(value: &String) -> Result<UserWrapper, InteractionError> {
    if value.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value.parse().unwrap()).await
    } else {
        UserWrapper::new_user_by_search(value).await
    }
}
