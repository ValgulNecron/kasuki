use std::fs;
use std::path::Path;

use image::io::Reader;
use image::{imageops, DynamicImage, Rgba};
use serenity::all::{ChannelId, Context, CreateAttachment, CreateMessage, Member, PartialGuild};
use uuid::Uuid;

use crate::command::run::command_dispatch::check_if_module_is_on;
use crate::constant::SERVER_IMAGE_PATH;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::new_member::{load_localization_new_member, NewMemberLocalised};
use crate::background_task::server_image::calculate_user_color::get_image_from_url;

/// Main function to handle new member joining the server
///
/// # Arguments
///
/// * `ctx` - A context instance from the serenity crate
/// * `member` - A mutable reference to the new member
///
/// # Returns
///
/// * `Result<(), AppError>` - An empty result indicating success or an error
pub async fn new_member(ctx: Context, member: &mut Member) -> Result<(), AppError> {
    // Create directory if it doesn't exist
    create_dir()?;
    // Get the guild ID
    let guild_id = member.guild_id.to_string();
    // Check if the new member module is enabled
    check_new_member_module_status(guild_id.clone()).await?;

    // Get the path to the server image
    let full_image_path = format!("{}/{}.webp", SERVER_IMAGE_PATH, member.guild_id);
    // Load the localization for the new member
    let new_member_localised = load_localization_new_member(guild_id).await?;
    // Get the partial guild with counts
    let guild = member
        .guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| {
            AppError::new(
                format!("There was an error getting the guild. {}", e),
                ErrorType::Option,
                ErrorResponseType::None,
            )
        })?;
    // Define the dimensions of the image
    let dim_x = 4000;
    let dim_y = 1000;

    // Check if the image exists, if not use the default image
    let full_image_path = if Path::new(full_image_path.as_str()).exists() {
        full_image_path
    } else {
        format!("{}/default.webp", SERVER_IMAGE_PATH)
    };

    // Create the overlay image
    let bg_image = create_overlay(full_image_path, member, dim_x, dim_y).await?;
    // Create the welcome message
    let create_message = create_welcome_message(&new_member_localised, member);
    // Send the welcome message
    send_welcome_message(&ctx, &guild, bg_image, create_message).await?;

    Ok(())
}

/// Function to get the channel to send the welcome message
///
/// # Arguments
///
/// * `guild` - A partial guild instance
///
/// # Returns
///
/// * `Result<ChannelId, AppError>` - The ID of the channel or an error
async fn get_channel_to_send(guild: &PartialGuild) -> Result<ChannelId, AppError> {
    guild.system_channel_id.ok_or(AppError::new(
        String::from("There is no system channel"),
        ErrorType::Option,
        ErrorResponseType::None,
    ))
}

/// Function to create the directory if it doesn't exist
///
/// # Returns
///
/// * `Result<(), AppError>` - An empty result indicating success or an error
fn create_dir() -> Result<(), AppError> {
    if !Path::new(SERVER_IMAGE_PATH).exists() {
        fs::create_dir_all(SERVER_IMAGE_PATH).map_err(|e| {
            AppError::new(
                format!("Failed to create the directory. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })?;
    }

    Ok(())
}

/// Function to check if the new member module is enabled
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild
///
/// # Returns
///
/// * `Result<(), AppError>` - An empty result indicating success or an error
async fn check_new_member_module_status(guild_id: String) -> Result<(), AppError> {
    if !check_if_module_is_on(guild_id, "NEW_MEMBER").await? {
        return Err(AppError::new(
            String::from("The module is off"),
            ErrorType::Module,
            ErrorResponseType::None,
        ));
    }

    Ok(())
}

/// Function to get the base image
///
/// # Arguments
///
/// * `full_image_path` - The path to the image
///
/// # Returns
///
/// * `Result<DynamicImage, AppError>` - The base image or an error
fn get_base_image(full_image_path: String) -> Result<DynamicImage, AppError> {
    Reader::open(full_image_path)
        .map_err(|e| {
            AppError::new(
                format!("There was an error when opening the image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })?
        .decode()
        .map_err(|e| {
            AppError::new(
                format!("There was an error when decoding the image. {}", e),
                ErrorType::File,
                ErrorResponseType::None,
            )
        })
}

/// Function to get the overlay image
///
/// # Arguments
///
/// * `member` - A mutable reference to the new member
///
/// # Returns
///
/// * `Result<DynamicImage, AppError>` - The overlay image or an error
async fn get_overlay_image(member: &mut Member) -> Result<DynamicImage, AppError> {
    let mut overlay_image = get_image_from_url(member.face().replace("?size=1024", "?size=128"))
        .await?
        .to_rgba8();
    let radius = 64;

    // Create a circular mask for the overlay image
    overlay_image
        .enumerate_pixels_mut()
        .for_each(|(x, y, pixel)| {
            if distance_from_center(x as i32, y as i32, radius, radius) > radius as f32 {
                *pixel = Rgba([pixel[0], pixel[1], pixel[2], 0]);
            }
        });

    Ok(DynamicImage::from(overlay_image))
}

/// Function to calculate the distance from the center of the image
///
/// # Arguments
///
/// * `x` - The x-coordinate of the pixel
/// * `y` - The y-coordinate of the pixel
/// * `center_x` - The x-coordinate of the center of the image
/// * `center_y` - The y-coordinate of the center of the image
///
/// # Returns
///
/// * `f32` - The distance from the center of the image
fn distance_from_center(x: i32, y: i32, center_x: i32, center_y: i32) -> f32 {
    let dx = x - center_x;
    let dy = y - center_y;
    ((dx * dx + dy * dy) as f32).sqrt()
}

/// Function to create the overlay image
///
/// # Arguments
///
/// * `full_image_path` - The path to the full image
/// * `member` - A mutable reference to the new member
/// * `dim_x` - The width of the image
/// * `dim_y` - The height of the image
///
/// # Returns
///
/// * `Result<DynamicImage, AppError>` - The overlay image or an error
async fn create_overlay(
    full_image_path: String,
    member: &mut Member,
    dim_x: i32,
    dim_y: i32,
) -> Result<DynamicImage, AppError> {
    let mut bg_image = get_base_image(full_image_path)?;
    let overlay_image = get_overlay_image(member).await?;

    let offset_x = (dim_x / 2) - (128 / 2);
    let offset_y = (dim_y / 2) - (128 / 2);
    imageops::overlay(
        &mut bg_image,
        &overlay_image,
        offset_x as i64,
        offset_y as i64,
    );

    Ok(bg_image)
}

/// Function to create the welcome message
///
/// # Arguments
///
/// * `new_member_localised` - A reference to the localized new member message
/// * `member` - A reference to the new member
///
/// # Returns
///
/// * `CreateMessage` - The welcome message
fn create_welcome_message(
    new_member_localised: &NewMemberLocalised,
    member: &Member,
) -> CreateMessage {
    let mut create_message = CreateMessage::default();
    create_message = create_message.content(
        new_member_localised
            .welcome
            .replace("$user$", &format!("<@{}>", member.user.id)),
    );

    create_message
}

/// Function to send the welcome message
///
/// # Arguments
///
/// * `ctx` - A reference to the context instance
/// * `guild` - A reference to the partial guild
/// * `bg_image` - The background image
/// * `create_message` - The welcome message
///
/// # Returns
///
/// * `Result<(), AppError>` - An empty result indicating success or an error
async fn send_welcome_message(
    ctx: &Context,
    guild: &PartialGuild,
    bg_image: DynamicImage,
    create_message: CreateMessage,
) -> Result<(), AppError> {
    let path = format!("{}.png", Uuid::new_v4());
    let channel = get_channel_to_send(guild).await?;
    let attachment = CreateAttachment::bytes(bg_image.as_bytes(), &path);
    let create_message = create_message.add_file(attachment);
    channel
        .send_message(&ctx.http, create_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("There was an error sending the message. {}", e),
                ErrorType::NewMember,
                ErrorResponseType::None,
            )
        })?;

    Ok(())
}
