use std::io::Cursor;
use std::time::Duration;

use crate::database::dispatcher::data_dispatch::{
    get_user_approximated_color, set_user_approximated_color,
};
use crate::database_struct::user_color::UserColor;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use base64::engine::general_purpose;
use base64::Engine;
use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use serenity::all::{Context, GuildId, Member, UserId};
use tokio::time::sleep;
use tracing::{debug, error};

pub async fn calculate_users_color(members: Vec<Member>) -> Result<(), AppError> {
    for member in members {
        let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://cdn.discordapp.com/avatars/260706120086192129/ec231a35c9a33dd29ea4819d29d06056.webp?size=64"))
            .replace("?size=1024", "?size=64");

        let id = member.user.id.to_string();

        let user_color: UserColor = get_user_approximated_color(&id).await?;
        let pfp_url_old = user_color.pfp_url.clone();
        if pfp_url != pfp_url_old.unwrap_or_default() {
            let (average_color, image): (String, String) = calculate_user_color(member).await?;
            set_user_approximated_color(&id, &average_color, &pfp_url, &image).await?
        }
        sleep(Duration::from_millis(100)).await
    }
    Ok(())
}

pub async fn return_average_user_color(
    members: Vec<Member>,
) -> Result<Vec<(String, String, String)>, AppError> {
    let mut average_colors = Vec::new();
    for member in members {
        let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://cdn.discordapp.com/avatars/260706120086192129/ec231a35c9a33dd29ea4819d29d06056.webp?size=64"))
            .replace("?size=1024", "?size=64");
        let id = member.user.id.to_string();

        let user_color: UserColor = get_user_approximated_color(&id).await?;
        let color = user_color.color.clone();
        let pfp_url_old = user_color.pfp_url.clone();
        let image_old = user_color.image;
        if pfp_url != pfp_url_old.clone().unwrap_or_default() {
            let (average_color, image): (String, String) = calculate_user_color(member).await?;
            set_user_approximated_color(&id, &average_color, &pfp_url, &image).await?;
            average_colors.push((average_color, pfp_url, image))
        } else {
            average_colors.push((color.unwrap(), pfp_url_old.unwrap(), image_old.unwrap()))
        }
    }

    Ok(average_colors)
}

async fn calculate_user_color(member: Member) -> Result<(String, String), AppError> {
    let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://cdn.discordapp.com/avatars/260706120086192129/ec231a35c9a33dd29ea4819d29d06056.webp?size=64"))
        .replace("?size=1024", "?size=64");

    let img = get_image_from_url(pfp_url).await?;

    // convert to rgba8 so every image use the same color type.
    let img = img.to_rgba8();

    // Fallback to CPU multithreading with rayon
    let (r_total, g_total, b_total) = img
        .enumerate_pixels()
        .par_bridge()
        .map(|(_, _, pixel)| (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32))
        .reduce(
            || (0, 0, 0),
            |(r1, g1, b1), (r2, g2, b2)| (r1 + r2, g1 + g2, b1 + b2),
        );

    debug!("R: {}, G: {}, B: {}", r_total, g_total, b_total);

    // Calculate the average color by dividing the sum by the total number of pixels
    let num_pixels = img.width() * img.height();
    let r_avg = r_total / num_pixels;
    let g_avg = g_total / num_pixels;
    let b_avg = b_total / num_pixels;

    let average_color = format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg);
    debug!("{}", average_color);

    let mut image_data: Vec<u8> = Vec::new();
    PngEncoder::new(&mut image_data)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgba8,
        )
        .unwrap();

    let base64_image = general_purpose::STANDARD.encode(image_data.clone());
    let image = format!("data:image/png;base64,{}", base64_image);
    // Return the average color
    Ok((average_color, image))
}

pub async fn get_image_from_url(url: String) -> Result<DynamicImage, AppError> {
    // Fetch the image data
    let resp = reqwest::get(url)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to download image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })?
        .bytes()
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to get bytes image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })?;

    // Decode the image data
    let img = ImageReader::new(Cursor::new(resp))
        .with_guessed_format()
        .map_err(|e| {
            AppError::new(
                format!("Failed to load image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })?
        .decode()
        .map_err(|e| {
            AppError::new(
                format!("Failed to decode image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })?;

    Ok(img)
}

pub async fn color_management(guilds: &Vec<GuildId>, ctx_clone: &Context) {
    let mut members: Vec<Member> = Vec::new();
    let guild_len = guilds.len();
    debug!(guild_len);
    for guild in guilds {
        let guild_id = guild.to_string();
        debug!(guild_id);
        let mut members_temp_out = get_member(ctx_clone, guild).await;
        let server_member_len = members_temp_out.len();
        debug!(server_member_len);
        members.append(&mut members_temp_out);
        let members_len = members.len();
        debug!(members_len);
    }
    match calculate_users_color(members.into_iter().collect()).await {
        Ok(_) => {}
        Err(e) => error!("{:?}", e),
    };
}

pub async fn get_member(ctx_clone: &Context, guild: &GuildId) -> Vec<Member> {
    let mut i = 0;
    let mut members_temp_out: Vec<Member> = Vec::new();
    while members_temp_out.len() == (1000 * i) {
        let mut members_temp_in = if i == 0 {
            guild
                .members(&ctx_clone.http, Some(1000), None)
                .await
                .unwrap()
        } else {
            let user: UserId = members_temp_out.last().unwrap().user.id;
            guild
                .members(&ctx_clone.http, Some(1000), Some(user))
                .await
                .unwrap()
        };
        i += 1;
        members_temp_out.append(&mut members_temp_in);
    }
    members_temp_out
}
