use serenity::all::{
    ComponentInteraction, Context, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EditInteractionResponse,
    EditMessage, Timestamp,
};
use tracing::trace;

use crate::constant::{ACTIVITY_LIST_LIMIT, COLOR};
use crate::database::data_struct::server_activity::ServerActivity;
use crate::database::manage::dispatcher::data_dispatch::get_all_server_activity;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;

/// Updates the activity list in the server.
///
/// This function takes a context, a component interaction, and a page number as parameters.
/// It retrieves the guild ID from the component interaction and loads the localized activity list.
/// It then retrieves all server activities and formats them into a list.
/// The function creates an embed message with the activity list and updates the message with the embed.
/// If there are more activities than the limit, it adds a button to the message to go to the next page.
///
/// # Arguments
///
/// * `ctx` - A reference to the context.
/// * `component_interaction` - A reference to the component interaction.
/// * `page_number` - A string that represents the current page number.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn update(
    ctx: &Context,
    component_interaction: &ComponentInteraction,
    page_number: &str,
) -> Result<(), AppError> {
    let guild_id = match component_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_activity_localised_text = load_localization_list_activity(guild_id).await?;

    let guild_id = component_interaction.guild_id.ok_or(AppError::new(
        String::from("There is no guild id"),
        ErrorType::Option,
        ErrorResponseType::None,
    ))?;

    let list = get_all_server_activity(&guild_id.to_string()).await?;
    let len = list.len();
    let actual_page: u64 = page_number.parse().unwrap();
    trace!("{:?}", actual_page);
    let next_page: u64 = actual_page + 1;
    let previous_page: u64 = if actual_page > 0 { actual_page - 1 } else { 0 };

    let activity: Vec<String> = get_formatted_activity_list(list, actual_page);

    let join_activity = activity.join("\n");

    let builder_message = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(list_activity_localised_text.title)
        .description(join_activity);
    let mut message_rep = CreateInteractionResponseMessage::new().embed(builder_message);

    if page_number != "0" {
        message_rep = message_rep.button(
            CreateButton::new(format!("next_activity_{}", previous_page))
                .label(&list_activity_localised_text.previous),
        );
    }
    trace!("{:?}", len);
    trace!("{:?}", ACTIVITY_LIST_LIMIT);
    if len > ACTIVITY_LIST_LIMIT as usize
        && (len > (ACTIVITY_LIST_LIMIT * (actual_page + 1)) as usize)
    {
        message_rep = message_rep.button(
            CreateButton::new(format!("next_activity_{}", next_page))
                .label(&list_activity_localised_text.next),
        )
    }
    let response = CreateInteractionResponse::UpdateMessage(message_rep);

    component_interaction
        .create_response(&ctx.http, response)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {:?}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    Ok(())
}

/// Formats a list of server activities into a list of strings.
///
/// This function takes a list of server activities and a page number as parameters.
/// It iterates over the list and formats each activity into a string.
/// It then skips the activities that are not on the current page and takes the activities that are on the current page.
///
/// # Arguments
///
/// * `list` - A vector of ServerActivity.
/// * `actual_page` - A u64 that represents the current page number.
///
/// # Returns
///
/// * A vector of strings where each string represents a formatted server activity.
pub fn get_formatted_activity_list(list: Vec<ServerActivity>, actual_page: u64) -> Vec<String> {
    list.into_iter()
        .map(|activity| {
            let anime_id = activity.anime_id;
            let name = activity.name;
            format!(
                "[{}](https://anilist_user.co/anime/{})",
                name.unwrap_or_default(),
                anime_id.unwrap_or_default()
            )
        })
        .skip((ACTIVITY_LIST_LIMIT * actual_page) as usize)
        .take(ACTIVITY_LIST_LIMIT as usize)
        .collect()
}
