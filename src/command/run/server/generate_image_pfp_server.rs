use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponseMessage,
};
use serenity::builder::CreateInteractionResponseFollowup;
use tracing::trace;
use uuid::Uuid;

use crate::config::Config;
use crate::database::manage::dispatcher::data_dispatch::get_server_image;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::server::generate_image_pfp_server::load_localization_pfp_server_image;

/// Executes the command to send an embed with the server's profile picture.
///
/// This function calls the `send_embed` function with the `image_type` set to "local".
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    let db_type = config.bot.config.db_type.clone();
    send_embed(ctx, command_interaction, "local", db_type).await
}

/// Sends an embed with the server's profile picture.
///
/// This function retrieves the server's profile picture, decodes it from base64, and sends it as an embed in a response to the command interaction.
/// The embed includes the title and the image of the server's profile picture.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `image_type` - The type of the image to be sent. This is used to determine which image to retrieve from the server.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    image_type: &str,
    db_type: String,
) -> Result<(), AppError> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized text for the server's profile picture image
    let pfp_server_image_localised_text =
        load_localization_pfp_server_image(guild_id.clone(), db_type.clone()).await?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;

    // Retrieve the server's profile picture image
    let image = get_server_image(&guild_id, &image_type.to_string(), db_type)
        .await?
        .1
        .unwrap_or_default();

    // Decode the image from base64
    let input = image.trim_start_matches("data:image/png;base64,");
    let image_data: Vec<u8> = BASE64.decode(input).map_err(|e| {
        AppError::new(
            format!("Error when decoding the image or there is no image {}", e),
            ErrorType::Option,
            ErrorResponseType::Message,
        )
    })?;

    // Generate a unique filename for the image
    let uuid = Uuid::new_v4();
    let image_path = format!("{}.png", uuid);

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    // Create an attachment with the image
    let attachment = CreateAttachment::bytes(image_data, image_path);

    // Construct the follow-up response with the embed and the attachment
    let builder = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    // Send the follow-up response to the command interaction
    command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;
    trace!("Done");
    Ok(())
}
