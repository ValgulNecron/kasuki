use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::get_all_user_approximated_color;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{DifferedError, Error};
use crate::error_enum::CommandError::{ErrorCommandSendingError, ErrorOptionError};
use crate::error_enum::DifferedCommandError::{DifferedCommandSendingError, WritingFile};
use crate::image_saver::general_image_saver::image_saver;
use crate::lang_struct::general::generate_image_pfp_server::load_localization_pfp_server_image;
use crate::server_image::calculate_user_color::get_image_from_url;
use crate::server_image::common::create_color_vector_from_user_color;

use crate::database::sqlite::data::get_server_image_sqlite;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageEncoder};
use log::trace;
use palette::{IntoColor, Lab, Srgb};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use tracing::{debug, error};
use uuid::Uuid;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let guild = command_interaction
        .guild_id
        .unwrap()
        .to_partial_guild(&ctx.http)
        .await
        .map_err(|e| Error(ErrorOptionError(format!("There is no option {}", e))))?;

    let pfp_server_image_localised_text =
        load_localization_pfp_server_image(guild_id.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            Error(ErrorCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;

    let image = get_server_image_sqlite(&guild_id, &String::from("global"))
        .await?
        .1
        .unwrap_or_default();
    let input = image.trim_start_matches("data:image/png;base64,");
    let image_data: Vec<u8> = BASE64.decode(input).unwrap();
    let uuid = Uuid::new_v4();
    let image_path = format!("{}.png", uuid);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    let attachment = CreateAttachment::bytes(image_data, image_path);

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            DifferedError(DifferedCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;
    trace!("Done");
    Ok(())
}
