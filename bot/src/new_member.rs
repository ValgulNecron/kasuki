use crate::config::DbConfig;
use crate::constant::{HEX_COLOR, NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::structure::message::new_member::load_localization_new_member;
use image::ImageFormat::WebP;
use image::{DynamicImage, GenericImage};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, CreateMessage, GuildId, Http, Member, PartialGuild};
use serenity::builder::CreateAttachment;
use serenity::prelude::Context as SerenityContext;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;
use text_to_png::TextRenderer;
use tracing::{debug, info};

use anyhow::{Context, Result};

// Enums
#[derive(Debug)]

pub enum XAlignment {
    Center,
    Right,
}

#[derive(Debug)]

pub enum YAlignment {
    Center,
    Bottom,
}

// Structs
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct NewMemberSetting {
    pub custom_channel: bool,
    pub channel_id: u64,
    pub custom_image: bool,
    pub show_username: bool,
    pub show_time_join: bool,
}

impl Default for NewMemberSetting {
    fn default() -> Self {
        NewMemberSetting {
            custom_channel: false,
            channel_id: 0,
            custom_image: false,
            show_username: true,
            show_time_join: true,
        }
    }
}

// Helper functions
pub async fn load_guild_settings(guild_id: GuildId) -> NewMemberSetting {
    debug!("Loading guild settings for guild: {}", guild_id);

    let content = fs::read_to_string(NEW_MEMBER_PATH).unwrap_or_default();

    let settings_map: HashMap<String, NewMemberSetting> =
        serde_json::from_str(&content).unwrap_or_default();

    settings_map
        .get(&guild_id.to_string())
        .unwrap_or(&NewMemberSetting::default())
        .clone()
}

pub fn load_new_member_image(guild_id: String) -> Option<Vec<u8>> {
    debug!("Loading new member image for guild: {}", guild_id);

    let image_path = format!("{}{}.png", NEW_MEMBER_IMAGE_PATH, guild_id);

    fs::read(image_path).ok()
}

pub fn create_default_new_member_image() -> Result<Vec<u8>> {
    debug!("Creating default new member image");

    let width = 2000;

    let height = width / 4;

    let img = DynamicImage::new_rgba8(width, height);

    let mut bytes = Vec::new();

    img.write_to(&mut Cursor::new(&mut bytes), WebP)?;

    Ok(bytes)
}

pub fn get_server_image(guild_id: String, guild_settings: &NewMemberSetting) -> Option<Vec<u8>> {
    debug!("Getting server image for guild: {}", guild_id);

    if guild_settings.custom_image {
        load_new_member_image(guild_id)
    } else {
        create_default_new_member_image().ok()
    }
}

pub fn get_channel_id(
    guild_settings: &NewMemberSetting,
    partial_guild: &PartialGuild,
) -> Option<ChannelId> {
    debug!("Getting channel ID for guild");

    if guild_settings.custom_channel {
        Some(ChannelId::from(guild_settings.channel_id))
    } else {
        partial_guild.system_channel_id
    }
}

pub async fn get_image(avatar_url: String) -> Result<DynamicImage> {
    debug!("Fetching image from URL: {}", avatar_url);

    let client = reqwest::Client::new();

    let res = client.get(avatar_url).send().await?;

    let body = res.bytes().await?;

    image::load_from_memory(&body).context("Failed to load image from memory")
}

pub fn change_to_x256_url(url: String) -> String {
    debug!("Changing URL size to 64x64: {}", url);

    let mut url = url
        .replace("?size=4096", "?size=256")
        .replace("?size=2048", "?size=256")
        .replace("?size=1024", "?size=256")
        .replace("?size=512", "?size=256")
        .replace("?size=256", "?size=256")
        .replace("?size=128", "?size=256")
        .replace("?size=64", "?size=256");

    if !url.ends_with("?size=256") {
        url = format!("{}?size=256", url)
    }

    url
}

pub async fn send_image(
    channel_id: ChannelId,
    image_bytes: Vec<u8>,
    http: &Arc<Http>,
) -> Result<()> {
    debug!("Sending image to channel: {}", channel_id);

    let attachment = CreateAttachment::bytes(image_bytes, "new_member.webp");

    let message = CreateMessage::new().add_file(attachment);

    channel_id
        .send_message(http, message)
        .await
        .context("Failed to send message")?;

    Ok(())
}

pub fn encode_image(image: DynamicImage) -> Result<Vec<u8>> {
    debug!("Encoding image");

    let rgba8_image = image.to_rgba8();

    let mut buffer = Cursor::new(Vec::new());

    rgba8_image
        .write_to(&mut buffer, WebP)
        .context("Failed to write image to buffer")?;

    Ok(buffer.into_inner())
}

pub async fn overlay_image(
    background_image: &mut DynamicImage,
    foreground_image: DynamicImage,
) -> Result<(DynamicImage, u32, u32, u32, u32)> {
    debug!("Overlaying foreground image onto background image");

    let (background_width, background_height) =
        (background_image.width(), background_image.height());

    let (foreground_width, foreground_height) =
        (foreground_image.width(), foreground_image.height());

    let (x_offset, y_offset) = (
        (background_width - foreground_width) / 2,
        (background_height - foreground_height) / 2,
    );

    background_image
        .copy_from(&foreground_image, x_offset, y_offset)
        .context("Failed to copy foreground image")?;

    Ok((
        background_image.clone(),
        background_width,
        background_height,
        foreground_width,
        foreground_height,
    ))
}

pub async fn add_text(
    image: &mut DynamicImage,
    text: String,
    x_alignment: XAlignment,
    y_alignment: YAlignment,
    offset: u32,
) -> Result<DynamicImage> {
    debug!("Adding text to image: {}", text);

    let renderer = TextRenderer::default();

    let text_png = renderer
        .render_text_to_png_data(text, 52, HEX_COLOR)
        .context("Failed to render text")?;

    let text_image =
        image::load_from_memory(&text_png.data).context("Failed to load text image")?;

    let (text_image_width, text_image_height) = (text_image.width(), text_image.height());

    let (image_width, image_height) = (image.width(), image.height());

    let x = match x_alignment {
        XAlignment::Center => (image_width - text_image_width) / 2,
        XAlignment::Right => image_width - text_image_width + offset,
    };

    let y = match y_alignment {
        YAlignment::Center => (image_height - text_image_height) / 2,
        YAlignment::Bottom => image_height - text_image_height + offset,
    };

    image
        .copy_from(&text_image, x, y)
        .context("Failed to copy text image")?;

    Ok(image.clone())
}

// Main function
pub async fn new_member_message(
    ctx: &SerenityContext,
    member: &Member,
    db_config: DbConfig,
) -> Result<()> {
    info!(
        "Processing new member message for guild: {}",
        member.guild_id
    );

    let guild_id = member.guild_id;

    let guild_settings = load_guild_settings(guild_id).await;

    let channel_id = get_channel_id(
        &guild_settings,
        &guild_id.to_partial_guild(&ctx.http).await?,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to get the channel id"))?;

    let guild_image_data = get_server_image(guild_id.to_string(), &guild_settings)
        .ok_or_else(|| anyhow::anyhow!("Failed to get the server image"))?;

    let mut guild_image = image::load_from_memory(&guild_image_data)?;

    let avatar_url = change_to_x256_url(member.face());

    let avatar_image = get_image(avatar_url).await?;

    let (_, _, _, _, image_height) = overlay_image(&mut guild_image, avatar_image).await?;

    let welcome_text = load_localization_new_member(guild_id.to_string(), db_config)
        .await
        .map(|local| local.welcome)
        .unwrap_or_else(|_| "Welcome $user$".to_string())
        .replace("$user$", &member.user.name);

    if guild_settings.show_username {
        guild_image = add_text(
            &mut guild_image,
            welcome_text,
            XAlignment::Center,
            YAlignment::Center,
            image_height,
        )
        .await?;
    }

    if guild_settings.show_time_join {
        let join_date = member
            .joined_at
            .unwrap_or_default()
            .format("%m/%d/%Y %H:%M:%S")
            .to_string();

        guild_image = add_text(
            &mut guild_image,
            join_date,
            XAlignment::Right,
            YAlignment::Bottom,
            0,
        )
        .await?;
    }

    let bytes = encode_image(guild_image)?;

    send_image(channel_id, bytes, &ctx.http).await?;

    info!(
        "Successfully processed new member message for guild: {}",
        member.guild_id
    );

    Ok(())
}
