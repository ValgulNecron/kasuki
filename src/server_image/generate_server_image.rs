use crate::database::dispatcher::data_dispatch::{
    get_all_user_approximated_color, get_server_image, set_server_image,
};

use crate::error_enum::AppError;
use crate::error_enum::AppError::NotACommandError;
use crate::error_enum::NotACommandError::NotACommandOptionError;
use crate::image_saver::general_image_saver::image_saver;
use crate::server_image::calculate_user_color::{
    get_image_from_url, get_member, return_average_user_color,
};
use crate::server_image::common::{
    create_color_vector_from_tuple, create_color_vector_from_user_color, find_closest_color, Color,
    ColorWithUrl,
};
use base64::engine::general_purpose;
use base64::Engine;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageEncoder};
use palette::{IntoColor, Lab, Srgb};
use serenity::all::{Context, GuildId, Member};
use serenity::futures::stream::FuturesUnordered;
use serenity::futures::StreamExt;
use std::sync::{Arc, Mutex};
use std::thread;

use tokio::task;

use tracing::{error, info};
use uuid::Uuid;

pub async fn generate_local_server_image(ctx: &Context, guild_id: GuildId) -> Result<(), AppError> {
    let members: Vec<Member> = get_member(ctx, &guild_id).await;
    let average_colors = return_average_user_color(members).await?;
    let color_vec = create_color_vector_from_tuple(average_colors.clone());
    generate_server_image(ctx, guild_id, color_vec, String::from("local")).await
}

pub async fn generate_global_server_image(
    ctx: &Context,
    guild_id: GuildId,
) -> Result<(), AppError> {
    let average_colors = get_all_user_approximated_color().await?;
    let color_vec = create_color_vector_from_user_color(average_colors.clone());

    generate_server_image(ctx, guild_id, color_vec, String::from("global")).await
}

async fn generate_server_image(
    ctx: &Context,
    guild_id: GuildId,
    average_colors: Vec<ColorWithUrl>,
    image_type: String,
) -> Result<(), AppError> {
    let guild = guild_id.to_partial_guild(&ctx.http).await.map_err(|e| {
        NotACommandError(NotACommandOptionError(format!("There is no option {}", e)))
    })?;
    let guild_pfp = guild.icon_url().unwrap_or(String::from("https://imgs.search.brave.com/FhPP6x9omGE50_uLbcuizNYwrBLp3bQZ8ii9Eel44aQ/rs:fit:860:0:0/g:ce/aHR0cHM6Ly9pbWcu/ZnJlZXBpay5jb20v/ZnJlZS1waG90by9h/YnN0cmFjdC1zdXJm/YWNlLXRleHR1cmVz/LXdoaXRlLWNvbmNy/ZXRlLXN0b25lLXdh/bGxfNzQxOTAtODE4/OS5qcGc_c2l6ZT02/MjYmZXh0PWpwZw"))
        .replace("?size=1024", "?size=128");

    let old_url = get_server_image(&guild_id.to_string(), &image_type)
        .await?
        .0;

    if old_url.unwrap_or_default() == guild_pfp {
        return Ok(());
    }

    let img = get_image_from_url(guild_pfp.clone()).await?;

    let dim = 128 * 64;

    let mut handles = vec![];
    let mut combined_image = DynamicImage::new_rgba16(dim, dim);
    let vec_image = Arc::new(Mutex::new(Vec::new()));
    log::trace!("Started creation");
    for y in 0..img.height() {
        for x in 0..img.width() {
            let pixel = img.get_pixel(x, y);
            let color_vec_moved = average_colors.clone();
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

    let base64_image = general_purpose::STANDARD.encode(image_data.clone());
    let image = format!("data:image/png;base64,{}", base64_image);
    let uuid = Uuid::new_v4();
    image_saver(guild_id.to_string(), uuid.to_string(), image_data).await?;
    set_server_image(&guild_id.to_string(), &image_type, &image, &guild_pfp).await
}

pub async fn server_image_management(ctx: &Context) {
    let guilds = ctx.cache.guilds();
    let mut tasks = FuturesUnordered::new();

    for guild in guilds {
        let ctx_clone = ctx.clone(); // Clone the context if it's needed inside the async block.
        let guild_clone = guild; // Clone the guild if it's needed inside the async block.

        let local_server_task = task::spawn(async move {
            match generate_local_server_image(&ctx_clone, guild_clone).await {
                Ok(_) => info!("Generated local server image for guild {}", guild),
                Err(e) => error!(
                    "Failed to generate local server image for guild {}. {:?}",
                    guild, e
                ),
            }
        });

        let ctx_clone = ctx.clone(); // Clone the context if it's needed inside the async block.
        let guild_clone = guild; // Clone the guild if it's needed inside the async block.
        let global_server_task = task::spawn(async move {
            match generate_global_server_image(&ctx_clone, guild_clone).await {
                Ok(_) => info!("Generated global server image for guild {}", guild),
                Err(e) => error!(
                    "Failed to generate global server image for guild {}. {:?}",
                    guild, e
                ),
            }
        });

        tasks.push(local_server_task);
        tasks.push(global_server_task);
    }

    while let Some(_) = tasks.next().await {}
}
