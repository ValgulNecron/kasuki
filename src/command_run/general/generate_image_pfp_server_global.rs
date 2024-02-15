use crate::command_run::general::generate_image_pfp_server::{
    find_closest_color, get_color_with_url, Color, ColorWithUrl,
};
use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::get_all_user_approximated_color;
use crate::database_struct::user_color_struct::UserColor;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{DifferedError, Error};
use crate::error_enum::CommandError::{ErrorCommandSendingError, ErrorOptionError};
use crate::error_enum::DifferedCommandError::{DifferedCommandSendingError, WritingFile};
use crate::image_saver::general_image_saver::image_saver;
use crate::lang_struct::general::generate_image_pfp_server::load_localization_pfp_server_image;
use crate::server_image::calculate_user_color::get_image_from_url;
use crate::server_image::common::{
    create_color_vector_from_tuple, create_color_vector_from_user_color,
};
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
    let average_colors = get_all_user_approximated_color().await?;

    let guild_pfp = guild.icon_url().unwrap_or(String::from("https://imgs.search.brave.com/FhPP6x9omGE50_uLbcuizNYwrBLp3bQZ8ii9Eel44aQ/rs:fit:860:0:0/g:ce/aHR0cHM6Ly9pbWcu/ZnJlZXBpay5jb20v/ZnJlZS1waG90by9h/YnN0cmFjdC1zdXJm/YWNlLXRleHR1cmVz/LXdoaXRlLWNvbmNy/ZXRlLXN0b25lLXdh/bGxfNzQxOTAtODE4/OS5qcGc_c2l6ZT02/MjYmZXh0PWpwZw"))
        .replace("?size=1024", "?size=128");

    let img = get_image_from_url(guild_pfp).await?;

    let dim = 128 * 64;

    let color_vec = create_color_vector_from_user_color(average_colors.clone());
    let mut handles = vec![];
    let mut combined_image = DynamicImage::new_rgba16(dim, dim);
    let vec_image = Arc::new(Mutex::new(Vec::new()));
    trace!("Started creation");
    for y in 0..img.height() {
        for x in 0..img.width() {
            let pixel = img.get_pixel(x, y);
            let color_vec_moved = color_vec.clone();
            let vec_image_clone = Arc::clone(&vec_image); // Clone the Arc

            let handle = thread::spawn(move || {
                let vec_image = vec_image_clone;
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let r_normalized = r as f32 / 255.0;
                let g_normalized = g as f32 / 255.0;
                let b_normalized = b as f32 / 255.0;
                let rgb_color = Srgb::new(r_normalized, g_normalized, b_normalized);
                let lab_color: Lab = rgb_color.into_color();
                let color_target = Color { cielab: lab_color };
                let arr: &[ColorWithUrl] = &color_vec_moved;
                let closest_color = find_closest_color(arr, &color_target).unwrap();
                vec_image.lock().unwrap().push((x, y, closest_color.image));
            });

            handles.push(handle)
        }
    }
    for handle in handles {
        handle.join().unwrap()
    }
    let vec_image = vec_image.lock().unwrap().clone();
    for (x, y, image) in vec_image {
        combined_image.copy_from(&image, x * 64, y * 64).unwrap()
    }
    let image = combined_image.clone();
    let image = image::imageops::resize(
        &image,
        (4096.0 * 0.6) as u32,
        (4096.0 * 0.6) as u32,
        FilterType::CatmullRom,
    );
    let mut image_data: Vec<u8> = Vec::new();
    let encoder = PngEncoder::new(&mut image_data);
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8,
        )
        .unwrap();
    trace!("Created image");

    let combined_uuid = Uuid::new_v4();
    let image_path = &format!("{}.png", combined_uuid);
    fs::write(image_path, image_data.clone()).map_err(|e| {
        DifferedError(WritingFile(format!(
            "Failed to write the file bytes. {}",
            e
        )))
    })?;
    trace!("Saved image");

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    let attachment = CreateAttachment::path(&image_path).await.map_err(|e| {
        DifferedError(DifferedCommandSendingError(format!(
            "Error while uploading the attachment {}",
            e
        )))
    })?;

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

    image_saver(guild_id, image_path.clone(), image_data).await?;

    match fs::remove_file(image_path) {
        Ok(_) => debug!("File {} has been removed successfully", combined_uuid),
        Err(e) => error!("Failed to remove file {}: {}", combined_uuid, e),
    }
    Ok(())
}
