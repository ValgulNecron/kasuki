use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::trace;

use crate::helper::create_normalise_embed::get_default_embed;
use crate::components::anilist::list_all_activity::get_formatted_activity_list;
use crate::constant::ACTIVITY_LIST_LIMIT;
use crate::database::manage::dispatcher::data_dispatch::get_all_server_activity;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;

/// This asynchronous function runs the command interaction for transcribing an audio or video file.
///
/// It first retrieves the language and prompt for the transcription from the command interaction options.
/// It also retrieves the attachment to be transcribed.
///
/// It checks the content type of the attachment and returns an error if it is not an audio or video file.
///
/// It sends a deferred response to the command interaction.
///
/// It downloads the attachment and saves it to a local file.
///
/// It sends a request to the OpenAI API to transcribe the file.
///
/// It retrieves the transcription from the API response and sends a followup message with the transcription.
///
/// It handles any errors that occur during the process and returns an `AppError` if an error occurs.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_activity_localised_text = load_localization_list_activity(guild_id).await?;

    let guild_id = command_interaction.guild_id.ok_or(AppError::new(
        String::from("There is no guild id"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    let list = get_all_server_activity(&guild_id.to_string()).await?;
    let len = list.len();
    let next_page = 1;

    let activity: Vec<String> = get_formatted_activity_list(list, 0);

    let join_activity = activity.join("\n");

    let builder_message = get_default_embed(None)
        .title(list_activity_localised_text.title)
        .description(join_activity);

    let mut response = CreateInteractionResponseFollowup::new().embed(builder_message);
    trace!("{:?}", len);
    trace!("{:?}", ACTIVITY_LIST_LIMIT);
    if len > ACTIVITY_LIST_LIMIT as usize {
        response = response.button(
            CreateButton::new(format!("next_activity_{}", next_page))
                .label(&list_activity_localised_text.next),
        )
    }

    let _ = command_interaction
        .create_followup(&ctx.http, response)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;
    Ok(())
}
