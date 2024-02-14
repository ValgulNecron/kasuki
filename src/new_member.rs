use crate::command_run::command_dispatch::check_if_moule_is_on;
use crate::server_image::calculate_user_color::get_image_from_url;
use crate::constant::SERVER_IMAGE_PATH;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NewMemberError;
use crate::error_enum::NewMemberError::{
    NewMemberErrorOptionError, NewMemberFailedToCreateDirectory, NewMemberOff,
};
use crate::lang_struct::new_member::load_localization_new_member;
use image::io::Reader;
use image::{imageops, Rgba};
use serenity::all::{ChannelId, Context, CreateAttachment, CreateMessage, Member, PartialGuild};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub async fn new_member(ctx: Context, member: &mut Member) -> Result<(), AppError> {
    if !Path::new(SERVER_IMAGE_PATH).exists() {
        fs::create_dir_all(SERVER_IMAGE_PATH).map_err(|e| {
            NewMemberError(NewMemberFailedToCreateDirectory(format!(
                "Failed to create the directory {}",
                e
            )))
        })?;
    }

    let guild_id = member.guild_id.to_string();
    if !check_if_moule_is_on(guild_id.clone(), "NEW_MEMBER").await? {
        return Err(NewMemberError(NewMemberOff(String::from("it's off"))));
    }
    let new_member_localised = load_localization_new_member(guild_id).await?;

    let fip = format!("{}/{}.webp", SERVER_IMAGE_PATH, member.guild_id);
    let full_image_path = fip.as_str();

    let full_image_path = if Path::new(full_image_path).exists() {
        fip
    } else {
        format!("{}/default.webp", SERVER_IMAGE_PATH)
    };
    let guild = member
        .guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| {
            NewMemberError(NewMemberErrorOptionError(format!(
                "there was an error getting the guild. {}",
                e
            )))
        })?;
    let dim_x = 4000;
    let dim_y = 1000;
    let user_pfp = member.face().replace("?size=1024", "?size=128");
    let overlay_image = get_image_from_url(user_pfp).await?;
    let mut overlay_image = overlay_image.to_rgba8();
    let radius = 128 / 2;

    // Apply the round mask
    for (x, y, pixel) in overlay_image.enumerate_pixels_mut() {
        let dx = x as i32 - 128i32 / 2;
        let dy = y as i32 - 128i32 / 2;
        let distance = ((dx * dx + dy * dy) as f32).sqrt();

        if distance > radius as f32 {
            // Outside the circle, make it transparent
            *pixel = Rgba([pixel[0], pixel[1], pixel[2], 0]);
        }
    }
    let mut bg_image = Reader::open(full_image_path)
        .map_err(|e| {
            NewMemberError(NewMemberErrorOptionError(format!(
                "there was an error when opening the image. {}",
                e
            )))
        })?
        .decode()
        .map_err(|e| {
            NewMemberError(NewMemberErrorOptionError(format!(
                "there was an error when opening the image. {}",
                e
            )))
        })?;
    let offset_x = (dim_x / 2) - (128 / 2);
    let offset_y = (dim_y / 2) - (128 / 2);
    imageops::overlay(
        &mut bg_image,
        &overlay_image,
        offset_x as i64,
        offset_y as i64,
    );
    let uuid = Uuid::new_v4();
    let path = format!("{}.png", uuid);
    bg_image.save(&path).map_err(|e| {
        NewMemberError(NewMemberErrorOptionError(format!(
            "there was an error when creating the image. {}",
            e
        )))
    })?;

    let channel = get_channel_to_send(guild).await?;
    let attachment = CreateAttachment::path(&path).await.map_err(|e| {
        NewMemberError(NewMemberErrorOptionError(format!(
            "there was an error sending the image. {}",
            e
        )))
    })?;
    let mut create_message = CreateMessage::default();
    create_message = create_message.content(
        new_member_localised
            .welcome
            .replace("$user$", &format!("<@{}>", member.user.id)),
    );
    create_message = create_message.add_file(attachment);
    channel
        .send_message(&ctx.http, create_message)
        .await
        .map_err(|e| {
            NewMemberError(NewMemberErrorOptionError(format!(
                "there was an error sending the message. {}",
                e
            )))
        })?;

    fs::remove_file(path).map_err(|e| {
        NewMemberError(NewMemberErrorOptionError(format!(
            "Failed to remove the file. {}",
            e
        )))
    })?;

    Ok(())
}

async fn get_channel_to_send(guild: PartialGuild) -> Result<ChannelId, AppError> {
    guild
        .system_channel_id
        .ok_or(NewMemberError(NewMemberErrorOptionError(
            "there was an error getting the channel to send".to_string(),
        )))
}
