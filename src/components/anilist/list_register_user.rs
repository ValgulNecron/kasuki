use crate::command_run::anilist::list_register_user::get_the_list;
use crate::constant::{DIFFERED_COMMAND_SENDING_ERROR, MEMBER_LIMIT, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::list_register_user::load_localization_list_user;
use serenity::all::{
    ComponentInteraction, Context, CreateButton, CreateEmbed, EditMessage, UserId,
};
use tracing::trace;

pub async fn update(
    ctx: &Context,
    component_interaction: &ComponentInteraction,
    user_id: &str,
) -> Result<(), AppError> {
    let guild_id = match component_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_user_localised = load_localization_list_user(guild_id).await?;

    let guild_id = component_interaction.guild_id.ok_or(OPTION_ERROR.clone())?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let (builder_message, len, last_id): (CreateEmbed, usize, Option<UserId>) = get_the_list(
        guild,
        ctx,
        &list_user_localised,
        Some(user_id.parse().unwrap()),
    )
    .await?;

    let mut response = EditMessage::new().embed(builder_message);
    if len == MEMBER_LIMIT as usize {
        response = response.button(
            CreateButton::new(format!("next_{}", last_id.unwrap())).label(list_user_localised.next),
        )
    }

    let mut message = component_interaction.message.clone();

    let a = message.edit(&ctx.http, response).await;
    trace!("{:?}", a);
    a.map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())
}
