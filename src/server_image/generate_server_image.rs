use std::sync::{Arc, Mutex};
use std::thread;

use base64::Engine;
use base64::engine::general_purpose;
use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageEncoder};
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use palette::{IntoColor, Lab, Srgb};
use serenity::all::{Context, GuildId, Member};
use tokio::task;
use tracing::{error, info};
use uuid::Uuid;

use crate::database::dispatcher::data_dispatch::{
    get_all_user_approximated_color, set_server_image,
};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::image_saver::general_image_saver::image_saver;
use crate::server_image::calculate_user_color::{
    get_image_from_url, get_member, return_average_user_color,
};
use crate::server_image::common::{
    Color, ColorWithUrl, create_color_vector_from_tuple, create_color_vector_from_user_color,
    find_closest_color,
};

/// This function generates a local server image.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context struct provided by the serenity crate. This is used to interact with Discord's API.
/// * `guild_id` - The ID of the guild (server) for which the image is being generated.
///
/// # Returns
///
/// * `Result<(), AppError>` - This function returns a Result type. If the image generation is successful, it returns Ok(()), otherwise it returns an AppError.
///
/// # Errors
///
/// This function will return an error if there is a problem with fetching the members of the guild, calculating the average color, or generating the server image.
///
/// # Example
///
/// ```no_run
/// let ctx: Context = ...;
/// let guild_id: GuildId = ...;
/// let result = generate_local_server_image(&ctx, guild_id).await;
/// match result {
///     Ok(_) => println!("Image generated successfully"),
///     Err(e) => println!("Failed to generate image: {:?}", e),
/// }
/// ```
pub async fn generate_local_server_image(ctx: &Context, guild_id: GuildId) -> Result<(), AppError> {
    // Fetch the members of the guild
    let members: Vec<Member> = get_member(ctx, &guild_id).await;
    // Calculate the average color of the members' avatars
    let average_colors = return_average_user_color(members).await?;
    // Create a vector of colors from the average colors
    let color_vec = create_color_vector_from_tuple(average_colors.clone());
    // Generate the server image using the color vector and save it as a "local" image
    generate_server_image(ctx, guild_id, color_vec, String::from("local")).await
}

/// This function generates a global server image.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context struct provided by the serenity crate. This is used to interact with Discord's API.
/// * `guild_id` - The ID of the guild (server) for which the image is being generated.
///
/// # Returns
///
/// * `Result<(), AppError>` - This function returns a Result type. If the image generation is successful, it returns Ok(()), otherwise it returns an AppError.
///
/// # Errors
///
/// This function will return an error if there is a problem with fetching the approximated colors of all users, creating a color vector from user colors, or generating the server image.
///
/// # Example
///
/// ```no_run
/// let ctx: Context = ...;
/// let guild_id: GuildId = ...;
/// let result = generate_global_server_image(&ctx, guild_id).await;
/// match result {
///     Ok(_) => println!("Global image generated successfully"),
///     Err(e) => println!("Failed to generate global image: {:?}", e),
/// }
/// ```
pub async fn generate_global_server_image(
    ctx: &Context,
    guild_id: GuildId,
) -> Result<(), AppError> {
    let average_colors = get_all_user_approximated_color().await?;
    let color_vec = create_color_vector_from_user_color(average_colors.clone());
    generate_server_image(ctx, guild_id, color_vec, String::from("global")).await
}

/// This function generates a server image based on the average colors of the members' avatars.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context struct provided by the serenity crate. This is used to interact with Discord's API.
/// * `guild_id` - The ID of the guild (server) for which the image is being generated.
/// * `average_colors` - A vector of ColorWithUrl, which represents the average colors of the members' avatars.
/// * `image_type` - A string that represents the type of the image. It can be either "local" or "global".
///
/// # Returns
///
/// * `Result<(), AppError>` - This function returns a Result type. If the image generation is successful, it returns Ok(()), otherwise it returns an AppError.
///
/// # Errors
///
/// This function will return an error if there is a problem with fetching the guild, getting the guild's icon URL, getting the image from the URL, spawning threads, joining threads, copying from the image, resizing the image, writing the image, encoding the image, saving the image, or setting the server image.
pub async fn generate_server_image(
    ctx: &Context,
    guild_id: GuildId,
    average_colors: Vec<ColorWithUrl>,
    image_type: String,
) -> Result<(), AppError> {
    // Fetch the guild
    let guild = guild_id.to_partial_guild(&ctx.http).await.map_err(|e| {
        AppError::new(
            format!("Failed to get the guild. {}", e),
            ErrorType::Option,
            ErrorResponseType::None,
        )
    })?;
    // Get the guild's icon URL
    let guild_pfp = guild
        .icon_url()
        .ok_or(AppError::new(
            String::from("There is no option, no image for the guild."),
            ErrorType::Option,
            ErrorResponseType::None,
        ))?
        .replace("?size=1024", "?size=128");

    // Get the image from the URL
    let img = get_image_from_url(guild_pfp.clone()).await?;

    // Initialize the combined image
    let dim = 128 * 64;
    let mut combined_image = DynamicImage::new_rgba8(dim, dim);
    let vec_image = Arc::new(Mutex::new(Vec::new()));

    // For each pixel in the image, spawn a thread to find the closest color and push it to the vec_image
    for y in 0..img.height() {
        for x in 0..img.width() {
            let pixel = img.get_pixel(x, y);
            let color_vec_moved = average_colors.clone();
            let vec_image_clone = Arc::clone(&vec_image);

            let handle = thread::spawn(move || {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let r_normalized = r as f32 / 255.0;
                let g_normalized = g as f32 / 255.0;
                let b_normalized = b as f32 / 255.0;
                let rgb_color = Srgb::new(r_normalized, g_normalized, b_normalized);
                let lab_color: Lab = rgb_color.into_color();
                let color_target = Color { cielab: lab_color };
                let closest_color = find_closest_color(&color_vec_moved, &color_target).unwrap();
                vec_image_clone
                    .lock()
                    .unwrap()
                    .push((x, y, closest_color.image));
            });

            // Join the thread
            handle.join().unwrap();
        }
    }

    // Copy from the vec_image to the combined image
    let vec_image = vec_image.lock().unwrap().clone();
    for (x, y, image) in vec_image {
        combined_image.copy_from(&image, x * 64, y * 64).unwrap()
    }

    // Resize the combined image
    let image = image::imageops::resize(
        &combined_image,
        (4096.0 * 0.6) as u32,
        (4096.0 * 0.6) as u32,
        FilterType::CatmullRom,
    );

    let img = image;

    // Write the image
    let mut image_data: Vec<u8> = Vec::new();
    PngEncoder::new(&mut image_data)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgba8,
        )
        .unwrap();

    // Encode the image
    let base64_image = general_purpose::STANDARD.encode(image_data.clone());
    let image = format!("data:image/png;base64,{}", base64_image);
    let uuid = Uuid::new_v4();
    // Save the image
    image_saver(guild_id.to_string(), format!("{}.png", uuid), image_data).await?;
    // Set the server image
    set_server_image(&guild_id.to_string(), &image_type, &image, &guild_pfp).await
}

/// This function manages the generation of server images for all guilds in the cache.
///
/// It iterates over all guilds in the cache and spawns a new task for each guild to generate a local server image.
/// If the generation of the local server image fails, it logs an error message.
/// If the generation of the local server image is successful, it logs a success message.
///
/// After the local server image generation, it attempts to generate a global server image for the guild.
/// If the generation of the global server image fails, it logs an error message.
/// If the generation of the global server image is successful, it logs a success message.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context struct provided by the serenity crate. This is used to interact with Discord's API.
///
/// # Example
///
/// ```no_run
/// let ctx: Context = ...;
/// server_image_management(&ctx).await;
/// ```
pub async fn server_image_management(ctx: &Context) {
    for guild in ctx.cache.guilds() {
        let ctx_clone = ctx.clone();
        let guild_clone = guild;
        task::spawn(async move {
            if let Err(e) = generate_local_server_image(&ctx_clone, guild_clone).await {
                error!(
                    "Failed to generate local server image for guild {}. {:?}",
                    guild, e
                );
            } else {
                info!("Generated local server image for guild {}", guild);
            }
        });

        if let Err(e) = generate_global_server_image(ctx, guild).await {
            error!(
                "Failed to generate global server image for guild {}. {:?}",
                guild, e
            );
        } else {
            info!("Generated global server image for guild {}", guild);
        }
    }
}
