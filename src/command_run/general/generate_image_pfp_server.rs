use crate::common::calculate_user_color::return_average_user_color;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    CreatingImageError, DecodingImageError, DifferedWritingFile, FailedToGetImage,
};
use crate::lang_struct::general::generate_image_pfp_server::load_localization_pfp_server_image;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImage, GenericImageView};
use log::trace;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Member, Timestamp, UserId,
};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use image::imageops::FilterType;
use uuid::Uuid;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let pfp_server_image_localised_text = load_localization_pfp_server_image(guild_id).await?;

    let guild = command_interaction
        .guild_id
        .unwrap()
        .to_partial_guild(&ctx.http)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    let mut i = 0;
    let mut members: Vec<Member> = Vec::new();
    while members.len() == (1000 * i) {
        let mut members_temp = if i == 0 {
            guild.members(&ctx.http, Some(1000), None).await.unwrap()
        } else {
            let user: UserId = members.last().unwrap().user.id.clone();
            guild
                .members(&ctx.http, Some(1000), Some(user))
                .await
                .unwrap()
        };
        members.append(&mut members_temp);
        i += 1
    }

    let average_colors = return_average_user_color(members).await?;

    let guild_pfp = guild.icon_url().unwrap_or(String::from("https://imgs.search.brave.com/FhPP6x9omGE50_uLbcuizNYwrBLp3bQZ8ii9Eel44aQ/rs:fit:860:0:0/g:ce/aHR0cHM6Ly9pbWcu/ZnJlZXBpay5jb20v/ZnJlZS1waG90by9h/YnN0cmFjdC1zdXJm/YWNlLXRleHR1cmVz/LXdoaXRlLWNvbmNy/ZXRlLXN0b25lLXdh/bGxfNzQxOTAtODE4/OS5qcGc_c2l6ZT02/MjYmZXh0PWpwZw"))
        .replace("?size=1024", "?size=128");
    // Fetch the image data
    let resp = reqwest::get(guild_pfp)
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

    let dim = 128 * 32;

    let color_vec = create_color_vector(average_colors.clone());
    let mut handles = vec![];
    let combined_image = Arc::new(Mutex::new(DynamicImage::new_rgba16(dim, dim)));
    trace!("Started creation");
    for y in 0..img.height() {
        for x in 0..img.width() {
            let pixel = img.get_pixel(x, y);
            let color_vec_moved = color_vec.clone();
            let combined_image_clone = Arc::clone(&combined_image); // Clone the Arc

            let handle = thread::spawn(move || {
                let combined_image = combined_image_clone;
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                let color_target = Color { r, g, b, hex };
                let closest_color = find_closest_color(&color_vec_moved, &color_target).unwrap();

                combined_image
                    .lock()
                    .unwrap()
                    .copy_from(&closest_color.image, x * 32, y * 32)
                    .unwrap();
            });

            handles.push(handle);
        }
    }
    for handle in handles {
        handle.join().unwrap();
    }
    trace!("Created image");
    let image = combined_image.lock().unwrap().clone();

    let resized_img =
        image::imageops::resize(&image, 1024, 1024, FilterType::CatmullRom);

    let combined_uuid = Uuid::new_v4();
    let image_path = &format!("{}.png", combined_uuid);
    resized_img
        .save(&image_path)
        .map_err(|_| DifferedWritingFile(String::from("Failed to write the file bytes.")))?;
    trace!("Saved image");

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    let attachement = CreateAttachment::path(&image_path)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachement]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;
    trace!("Done");
    Ok(())
}

fn color_distance(color1: &ColorWithUrl, color2: &Color) -> f32 {
    ((color1.r as i32 - color2.r as i32).pow(2)
        + (color1.g as i32 - color2.g as i32).pow(2)
        + (color1.b as i32 - color2.b as i32).pow(2)) as f32
}

#[derive(Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    hex: String,
}

#[derive(Clone)]
struct ColorWithUrl {
    r: u8,
    g: u8,
    b: u8,
    hex: String,
    url: String,
    image: DynamicImage,
}

fn create_color_vector(tuples: Vec<(String, String, String)>) -> Vec<ColorWithUrl> {
    tuples
        .into_iter()
        .filter_map(|(hex, url, image)| {
            let r = hex[1..3].parse::<u8>();
            let g = hex[3..5].parse::<u8>();
            let b = hex[5..7].parse::<u8>();

            let input = image.trim_start_matches("data:image/png;base64,");
            let decoded = BASE64.decode(input).unwrap();
            let img = image::load_from_memory(&decoded).unwrap();

            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => Some(ColorWithUrl {
                    r,
                    g,
                    b,
                    hex,
                    url,
                    image: img,
                }),
                _ => None,
            }
        })
        .collect()
}

fn find_closest_color(colors: &Vec<ColorWithUrl>, target: &Color) -> Option<ColorWithUrl> {
    let mut min_distance = f32::MAX;
    let mut closest_color = None;
    for color in colors {
        let distance = color_distance(color, target);
        if distance < min_distance {
            min_distance = distance;
            closest_color = Some(color);
        }
    }
    closest_color.cloned()
}
