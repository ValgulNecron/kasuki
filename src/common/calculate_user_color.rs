use crate::error_enum::AppError;
use crate::error_enum::AppError::{CreatingImageError, DecodingImageError, FailedToGetImage};
use crate::sqls::general::data::{get_user_approximated_color, set_user_approximated_color};
use image::io::Reader as ImageReader;
use image::GenericImageView;
use log::trace;
use serenity::all::Member;
use std::io::Cursor;

pub async fn calculate_users_color(members: Vec<Member>) -> Result<(), AppError> {
    for member in members {
        let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://imgs.search.brave.com/FhPP6x9omGE50_uLbcuizNYwrBLp3bQZ8ii9Eel44aQ/rs:fit:860:0:0/g:ce/aHR0cHM6Ly9pbWcu/ZnJlZXBpay5jb20v/ZnJlZS1waG90by9h/YnN0cmFjdC1zdXJm/YWNlLXRleHR1cmVz/LXdoaXRlLWNvbmNy/ZXRlLXN0b25lLXdh/bGxfNzQxOTAtODE4/OS5qcGc_c2l6ZT02/MjYmZXh0PWpwZw"))
        .replace("?size=1024", "?size=32");

        let id = member.user.id.to_string();

        let (_, _, pfp_url_old): (Option<String>, Option<String>, Option<String>) =
            get_user_approximated_color(&id).await?;
        if pfp_url != pfp_url_old.unwrap_or(String::new()) {
            let average_color = calculate_user_color(member).await?;
            set_user_approximated_color(&id, &average_color, &pfp_url).await?
        }
    }
    Ok(())
}

pub async fn return_average_user_color(
    members: Vec<Member>,
) -> Result<Vec<(String, String)>, AppError> {
    let mut average_colors = Vec::new();
    for member in members {
        let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://imgs.search.brave.com/FhPP6x9omGE50_uLbcuizNYwrBLp3bQZ8ii9Eel44aQ/rs:fit:860:0:0/g:ce/aHR0cHM6Ly9pbWcu/ZnJlZXBpay5jb20v/ZnJlZS1waG90by9h/YnN0cmFjdC1zdXJm/YWNlLXRleHR1cmVz/LXdoaXRlLWNvbmNy/ZXRlLXN0b25lLXdh/bGxfNzQxOTAtODE4/OS5qcGc_c2l6ZT02/MjYmZXh0PWpwZw"))
            .replace("?size=1024", "?size=32");
        let id = member.user.id.to_string();

        let (_, color, pfp_url_old): (Option<String>, Option<String>, Option<String>) =
            get_user_approximated_color(&id).await?;
        if pfp_url != pfp_url_old.clone().unwrap_or(String::new()) {
            let average_color = calculate_user_color(member).await?;
            set_user_approximated_color(&id, &average_color, &pfp_url).await?;
            average_colors.push((average_color, pfp_url));
        } else {
            average_colors.push((color.unwrap(), pfp_url_old.unwrap()));
        }
    }

    Ok(average_colors)
}

async fn calculate_user_color(member: Member) -> Result<String, AppError> {
    let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://imgs.search.brave.com/FhPP6x9omGE50_uLbcuizNYwrBLp3bQZ8ii9Eel44aQ/rs:fit:860:0:0/g:ce/aHR0cHM6Ly9pbWcu/ZnJlZXBpay5jb20v/ZnJlZS1waG90by9h/YnN0cmFjdC1zdXJm/YWNlLXRleHR1cmVz/LXdoaXRlLWNvbmNy/ZXRlLXN0b25lLXdh/bGxfNzQxOTAtODE4/OS5qcGc_c2l6ZT02/MjYmZXh0PWpwZw"));

    // Fetch the image data
    let resp = reqwest::get(pfp_url)
        .await
        .map_err(|_| FailedToGetImage(String::from("Failed to download image.")))?
        .bytes()
        .await
        .map_err(|_| FailedToGetImage(String::from("Failed to get bytes image.")))?;

    // Decode the image data
    let img = ImageReader::new(Cursor::new(resp))
        .with_guessed_format()
        .map_err(|_| CreatingImageError(String::from("Failed to load image.")))?
        .decode()
        .map_err(|_| DecodingImageError(String::from("Failed to decode image.")))?;

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
    trace!("{}", average_color);

    // Return the average color
    Ok(average_color)
}
