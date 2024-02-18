use crate::command_run::anilist::list_register_user::get_the_list;
use crate::constant::MEMBER_LIST_LIMIT;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::list_register_user::load_localization_list_user;
use serenity::all::{
    ComponentInteraction, Context, CreateButton, CreateEmbed, EditMessage, UserId,
};
use tracing::trace;

pub async fn update(
    ctx: &Context,
    component_interaction: &ComponentInteraction,
    user_id: &str,
    prev_id: &str,
) -> Result<(), AppError> {
    let guild_id = match component_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_user_localised = load_localization_list_user(guild_id).await?;

    let guild_id = component_interaction
        .guild_id
        .ok_or(
            AppError::new(
                String::from("There is no guild id"),
                ErrorType::Option,
                ErrorResponseType::None,
            ))?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e|
            AppError::new(
                String::from("There is no guild"),
                ErrorType::Option,
                ErrorResponseType::None,
            ))?;

    let id = if user_id == "0" {
        None
    } else {
        Some(user_id.parse().unwrap())
    };

    let (builder_message, len, last_id): (CreateEmbed, usize, Option<UserId>) =
        get_the_list(guild, ctx, &list_user_localised, id).await?;

    let mut response = EditMessage::new().embed(builder_message);
    if user_id != "0" {
        response = response.button(
            CreateButton::new(format!("user_{}_{}", user_id, prev_id))
                .label(&list_user_localised.previous),
        );
    }
    if len > MEMBER_LIST_LIMIT as usize {
        response = response.button(
            CreateButton::new(format!("user_{}_{}", last_id.unwrap(), user_id))
                .label(list_user_localised.next),
        )
    }

    let mut message = component_interaction.message.clone();

    let a = message.edit(&ctx.http, response).await;
    trace!("{:?}", a);
    a.map_err(|e| {
        AppError::new(
            format!("Error while sending the component {}", e),
            ErrorType::Component,
            ErrorResponseType::None,
        )
    })
}
