use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{
    ComponentInteraction, Context, CreateButton, CreateEmbed, EditMessage, UserId,
};
use tokio::sync::RwLock;
use tracing::trace;

use crate::command::anilist_server::list_register_user::get_the_list;
use crate::config::BotConfigDetails;
use crate::constant::MEMBER_LIST_LIMIT;
use crate::helper::error_management::error_enum::UnknownResponseError;
use crate::structure::message::anilist_server::list_register_user::load_localization_list_user;

/// Updates the user list in the server.
///
/// This function takes a context, a component interaction, a user ID, and a previous ID as parameters.
/// It retrieves the guild ID from the component interaction and loads the localized user list.
/// It then retrieves all server users and formats them into a list.
/// The function creates an embed message with the user list and updates the message with the embed.
/// If there are more users than the limit, it adds a button to the message to go to the next page.
///
/// # Arguments
///
/// * `ctx` - A reference to the context.
/// * `component_interaction` - A reference to the component interaction.
/// * `user_id` - A string that represents the current user ID.
/// * `prev_id` - A string that represents the previous user ID.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn update(
    ctx: &Context,
    component_interaction: &ComponentInteraction,
    user_id: &str,
    prev_id: &str,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the guild ID from the component interaction
    let guild_id = match component_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized user list
    let list_user_localised = load_localization_list_user(guild_id, db_config.clone()).await?;

    // Retrieve the guild ID from the component interaction
    let guild_id = component_interaction
        .guild_id
        .ok_or(UnknownResponseError::Option(String::from(
            "Guild ID not found",
        )))?;

    // Retrieve the guild with counts
    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| UnknownResponseError::UserOrGuild(format!("{:#?}", e)))?;

    // Parse the user ID
    let id = if user_id == "0" {
        None
    } else {
        match user_id.parse() {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    };

    // Get the list of users
    let (builder_message, len, last_id): (CreateEmbed, usize, Option<UserId>) = get_the_list(
        guild,
        ctx,
        &list_user_localised,
        id,
        db_config,
    )
    .await?;

    // Create the response message
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

    // Clone the component interaction message
    let mut message = component_interaction.message.clone();

    // Edit the message with the response
    let a = message.edit(&ctx.http, response).await;
    trace!("{:?}", a);
    a.map_err(|e| UnknownResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
