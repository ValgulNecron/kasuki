use crate::database::dispatcher::data_dispatch::{
    get_user_approximated_color, set_user_approximated_color,
};
use crate::database_struct::user_color_struct::UserColor;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use base64::engine::general_purpose;
use base64::Engine;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, ImageOutputFormat};
use serenity::all::{Context, GuildId, Member, UserId};
use std::io::Cursor;
use std::time::Duration;
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

    // Create variables to hold the sum of each color channel
    let mut r_total: u32 = 0;
    let mut g_total: u32 = 0;
    let mut b_total: u32 = 0;

    // Iterate over each pixel
    for y in 0..img.height() {
        for x in 0..img.width() {
            // Get the pixel color
            let pixel = img.get_pixel(x, y);
            // Add the color to the total
            r_total += pixel[0] as u32;
            g_total += pixel[1] as u32;
            b_total += pixel[2] as u32;
        }
    }

    // Calculate the average color by dividing the sum by the total number of pixels
    let num_pixels = img.width() * img.height();
    let r_avg = r_total / num_pixels;
    let g_avg = g_total / num_pixels;
    let b_avg = b_total / num_pixels;

    let average_color = format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg);
    debug!("{}", average_color);

    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)
        .unwrap();
    let res_base64 = general_purpose::STANDARD.encode(&image_data);
    let image = format!("data:image/png;base64,{}", res_base64);
    // Return the average color
    Ok((average_color, image))
}

pub async fn get_image_from_url(url: String) -> Result<DynamicImage, AppError> {
    // Fetch the image data
    let resp = reqwest::get(url)
        .await
        .map_err(|e|
            AppError::new(
                format!("Failed to download image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            ))?
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
        .map_err(|e|
            AppError::new(
                format!("Failed to load image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            ))?
        .decode()
        .map_err(|e|
            AppError::new(
                format!("Failed to decode image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            ))?;

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
