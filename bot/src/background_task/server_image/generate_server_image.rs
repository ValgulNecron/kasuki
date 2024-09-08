use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread;

use base64::engine::general_purpose;
use base64::Engine;
use image::codecs::png;
use image::codecs::png::{CompressionType, PngEncoder};
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageEncoder};
use palette::{IntoColor, Lab, Srgb};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{Context, GuildId, Member};
use tokio::task;
use tracing::{info, warn};
use uuid::Uuid;

use crate::background_task::server_image::calculate_user_color::{
    get_image_from_url, get_member, return_average_user_color,
};
use crate::background_task::server_image::common::{
    create_color_vector_from_tuple, create_color_vector_from_user_color, find_closest_color, Color,
    ColorWithUrl,
};
use crate::config::{DbConfig, ImageConfig};
use crate::constant::THREAD_POOL_SIZE;
use crate::get_url;
use crate::helper::error_management::error_dispatch;
use crate::helper::image_saver::general_image_saver::image_saver;
use crate::structure::database::prelude::{ServerImage, UserColor};
use crate::structure::database::server_image::{ActiveModel, Column};

pub async fn generate_local_server_image(
    ctx: &Context,
    guild_id: GuildId,
    image_config: ImageConfig,
    db_config: DbConfig,
) -> Result<(), Box<dyn Error>> {

    let members: Vec<Member> = get_member(ctx.clone(), guild_id).await;

    let average_colors = return_average_user_color(members, db_config.clone()).await?;

    let color_vec = create_color_vector_from_tuple(average_colors.clone());

    generate_server_image(
        ctx,
        guild_id,
        color_vec,
        String::from("local"),
        image_config,
        db_config,
    )
    .await
}

/// This function generates a global server image.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context struct provided by the serenity crate. This is used to interact with Discord's API.
/// * `guild_id` - The ID of the guild (server) for which the image is being generated.
/// * `db_type` - A String representing the database type.
/// * `image_config` - The ImageConfig struct that contains configuration options for the image.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - This function returns a Result type. If the image generation is successful, it returns Ok(()), otherwise it returns an error wrapped in a Box.

pub async fn generate_global_server_image(
    ctx: &Context,
    guild_id: GuildId,
    image_config: ImageConfig,
    db_config: DbConfig,
) -> Result<(), Box<dyn Error>> {

    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

    let average_colors = UserColor::find().all(&connection).await?;

    let color_vec = create_color_vector_from_user_color(average_colors.clone());

    generate_server_image(
        ctx,
        guild_id,
        color_vec,
        String::from("global"),
        image_config,
        db_config,
    )
    .await
}

pub async fn generate_server_image(
    ctx: &Context,
    guild_id: GuildId,
    average_colors: Vec<ColorWithUrl>,
    image_type: String,
    image_config: ImageConfig,
    db_config: DbConfig,
) -> Result<(), Box<dyn Error>> {

    info!("Generating server image for {}.", guild_id);

    // Fetch the guild information
    let guild = guild_id.to_partial_guild(&ctx.http).await?;

    // Retrieve and process the guild image
    let guild_pfp = guild
        .icon_url()
        .ok_or(error_dispatch::Error::Option(String::from(
            "The guild has no icon",
        )))?
        .replace("?size=4096", "?size=64")
        .replace("?size=2048", "?size=64")
        .replace("?size=1024", "?size=64")
        .replace("?size=512", "?size=64")
        .replace("?size=256", "?size=64")
        .replace("?size=128", "?size=64");

    let img = get_image_from_url(guild_pfp.clone()).await?;

    let dim = 128 * 64;

    let mut combined_image = DynamicImage::new_rgba8(dim, dim);

    let vec_image: Arc<RwLock<Vec<(u32, u32, DynamicImage)>>> = Arc::new(RwLock::new(Vec::new()));

    let mut handles = Vec::new();

    // Process image pixels in parallel
    for y in 0..img.height() {

        for x in 0..img.width() {

            let pixel = img.get_pixel(x, y);

            let color_vec_moved = average_colors.clone();

            let vec_image_clone = Arc::clone(&vec_image);

            let handle = thread::spawn(move || {

                let r = pixel[0] as f32 / 255.0;

                let g = pixel[1] as f32 / 255.0;

                let b = pixel[2] as f32 / 255.0;

                let rgb_color = Srgb::new(r, g, b);

                let lab_color: Lab = <palette::rgb::Rgb as IntoColor<Lab>>::into_color(rgb_color);

                let color_target = Color { cielab: lab_color };

                let closest_color = match find_closest_color(&color_vec_moved, &color_target) {
                    Some(color) => color,
                    None => return,
                };

                let mut guard = match vec_image_clone.write() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                guard.push((x, y, closest_color.image))
            });

            handles.push(handle);

            if handles.len() >= THREAD_POOL_SIZE {

                for handle in handles {

                    match handle.join() {
                        Ok(_) => {}
                        Err(_) => continue,
                    }
                }

                handles = Vec::new();
            }
        }
    }

    // Combine processed images
    let vec_image = match vec_image.read() {
        Ok(vec_image) => vec_image.clone(),
        Err(e) => return Err(Box::new(error_dispatch::Error::Option(e.to_string()))),
    };

    let internal_vec = vec_image.clone();

    for (x, y, image) in internal_vec {

        match combined_image.copy_from(&image, x * 64, y * 64) {
            Ok(_) => {}
            Err(_) => continue,
        }
    }

    // Resize the final image
    let image = image::imageops::resize(
        &combined_image,
        (4096.0 * 0.6) as u32,
        (4096.0 * 0.6) as u32,
        FilterType::CatmullRom,
    );

    let img = image;

    // Write the image
    let mut image_data: Vec<u8> = Vec::new();

    PngEncoder::new_with_quality(
        &mut image_data,
        CompressionType::Best,
        png::FilterType::Adaptive,
    )
    .write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        ExtendedColorType::Rgba8,
    )?;

    // Encode the image to base64
    let base64_image = general_purpose::STANDARD.encode(image_data.clone());

    let image = format!("data:image/png;base64,{}", base64_image);

    let uuid = Uuid::new_v4();

    // Save the image
    let token = image_config.token.clone().unwrap_or_default();

    let saver = image_config.save_server.clone().unwrap_or_default();

    let save_type = image_config.save_image.clone();

    image_saver(
        guild_id.to_string(),
        format!("{}.png", uuid),
        image_data,
        saver,
        token,
        save_type,
    )
    .await?;

    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

    ServerImage::insert(ActiveModel {
        server_id: Set(guild_id.to_string()),
        server_name: Set(guild.name),
        image_type: Set(image_type.clone()),
        image: Set(image.clone()),
        image_url: Set(guild_pfp.clone()),
        ..Default::default()
    })
    .on_conflict(
        sea_orm::sea_query::OnConflict::columns([Column::ImageType, Column::ServerId])
            .update_column(Column::Image)
            .update_column(Column::ImageUrl)
            .update_column(Column::GeneratedAt)
            .update_column(Column::ServerName)
            .to_owned(),
    )
    .exec(&connection)
    .await?;

    Ok(())
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

pub async fn server_image_management(
    ctx: &Context,
    image_config: ImageConfig,
    db_config: DbConfig,
) {

    for guild in ctx.cache.guilds() {

        let ctx_clone = ctx.clone();

        let guild_clone = guild;

        let image_config_a = image_config.clone();

        let db_config2 = db_config.clone();

        task::spawn(async move {

            if let Err(e) =
                generate_local_server_image(&ctx_clone, guild_clone, image_config_a, db_config2)
                    .await
            {

                warn!(
                    "Failed to generate local server image for guild {}. {:?}",
                    guild, e
                );
            } else {

                info!("Generated local server image for guild {}", guild);
            }
        });

        if let Err(e) =
            generate_global_server_image(ctx, guild, image_config.clone(), db_config.clone()).await
        {

            warn!(
                "Failed to generate global server image for guild {}. {:?}",
                guild, e
            );
        } else {

            info!("Generated global server image for guild {}", guild);
        }
    }
}
