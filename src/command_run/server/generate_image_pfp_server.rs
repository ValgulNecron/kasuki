use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseMessage,
    Timestamp,
};
use serenity::builder::CreateInteractionResponseFollowup;
use tracing::trace;
use uuid::Uuid;

use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::get_server_image;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::server::generate_image_pfp_server::load_localization_pfp_server_image;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    send_embed(ctx, command_interaction, "local").await
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    image_type: &str,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let pfp_server_image_localised_text =
        load_localization_pfp_server_image(guild_id.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

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

    let image = get_server_image(&guild_id, &image_type.to_string())
        .await?
        .1
        .unwrap_or_default();
    let input = image.trim_start_matches("data:image/png;base64,");
    let image_data: Vec<u8> = BASE64.decode(input).map_err(|e| {
        AppError::new(
            format!("Error when decoding the image or there is no image {}", e),
            ErrorType::Option,
            ErrorResponseType::Message,
        )
    })?;
    let uuid = Uuid::new_v4();
    let image_path = format!("{}.png", uuid);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    let attachment = CreateAttachment::bytes(image_data, image_path);

    let builder = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

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
