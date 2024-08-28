use crate::config::DbConfig;
use crate::constant::{HEX_COLOR, NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::structure::message::new_member::load_localization_new_member;
use anyhow::anyhow;
use image::ImageFormat::WebP;
use image::{DynamicImage, GenericImage};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, Http, Member, PartialGuild};
use serenity::builder::CreateAttachment;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;
use text_to_png::TextRenderer;
use tracing::trace;

pub async fn new_member_message(
    ctx: &Context,
    member: &Member,
    db_config: DbConfig,
) -> Result<(), Box<dyn Error>> {
    let ctx = ctx.clone();
    let user_name = member.user.name.clone();
    let guild_id = member.guild_id;
    let partial_guild = guild_id.to_partial_guild(&ctx.http).await?;
    let guild_settings = load_guild_settings(guild_id).await;
    trace!(?guild_settings);

    let channel_id = get_channel_id(&guild_settings, &partial_guild)
        .ok_or(anyhow!("Failed to get the channel id."))?;
    trace!(?channel_id);

    let guild_image = get_server_image(guild_id.to_string(), &guild_settings)
        .ok_or(anyhow!("Failed to get the server image."))?;

    let mut guild_image = image::load_from_memory(&guild_image)?;

    let avatar = change_to_x64_url(member.face());
    trace!(?avatar);

    let image = get_image(avatar).await?;

    let (mut guild_image, guild_image_width, guild_image_height, _, image_height) =
        overlay_image(&mut guild_image, image).await?;

    let local = match load_localization_new_member(guild_id.to_string(), db_config).await {
        Ok(local) => local.welcome,
        Err(_) => "Welcome $user$".to_string(),
    };
    let text = local.replace("$user$", &user_name);
    guild_image = add_text_under_pseudo(
        &mut guild_image,
        text,
        guild_image_width,
        guild_image_height,
        image_height,
    )
    .await?;

    let join_data = member.joined_at.unwrap_or_default();
    let join_data = join_data.format("%m/%d/%Y %H:%M:%S").to_string();
    guild_image = add_text_bottom_right(
        &mut guild_image,
        join_data,
        guild_image_width,
        guild_image_height,
    )
    .await?;

    let bytes = encode_image(guild_image)?;
    send_image(channel_id, bytes, &ctx.http).await?;

    Ok(())
}

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
pub async fn load_guild_settings(guild_id: GuildId) -> NewMemberSetting {
    let content = fs::read_to_string(NEW_MEMBER_PATH).unwrap_or_default();
    let settings_map: HashMap<String, NewMemberSetting> =
        serde_json::from_str(&content).unwrap_or_default();
    settings_map
        .get(&guild_id.to_string())
        .unwrap_or(&NewMemberSetting::default())
        .clone()
}

pub fn load_new_member_image(guild_id: String) -> Option<Vec<u8>> {
    let image_path = format!("{}{}.png", NEW_MEMBER_IMAGE_PATH, guild_id);
    fs::read(image_path).ok()
}

pub fn create_default_new_member_image() -> Result<Vec<u8>, Box<dyn Error>> {
    let width = 2000;
    let height = width / 4;
    let img = DynamicImage::new_rgba8(width, height);
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), WebP)?;
    Ok(bytes)
}

pub fn get_server_image(guild_id: String, guild_settings: &NewMemberSetting) -> Option<Vec<u8>> {
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
    if guild_settings.custom_channel {
        Option::from(ChannelId::from(guild_settings.channel_id))
    } else {
        partial_guild.system_channel_id
    }
}

pub fn encode_image(image: DynamicImage) -> Result<Vec<u8>, Box<dyn Error>> {
    let rgba8_image = image.to_rgba8();
    let mut buffer = Cursor::new(Vec::new());
    rgba8_image.write_to(&mut buffer, WebP)?;
    Ok(buffer.into_inner().clone())
}

pub async fn send_image(
    channel_id: ChannelId,
    image_bytes: Vec<u8>,
    http: &Arc<Http>,
) -> Result<(), Box<dyn Error>> {
    let attachment = CreateAttachment::bytes(image_bytes, "new_member.webp");
    let message = CreateMessage::new().add_file(attachment);
    channel_id.send_message(http, message).await?;
    Ok(())
}

pub fn change_to_x64_url(url: String) -> String {
    url.replace("?size=4096", "?size=64")
        .replace("?size=2048", "?size=64")
        .replace("?size=1024", "?size=64")
        .replace("?size=512", "?size=64")
        .replace("?size=256", "?size=64")
        .replace("?size=128", "?size=64")
}

pub async fn get_image(avatar: String) -> Result<DynamicImage, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client.get(avatar).send().await?;
    let body = res.bytes().await?;
    let image = image::load_from_memory(&body)?;
    Ok(image)
}

pub async fn overlay_image(
    guild_image: &mut DynamicImage,
    image: DynamicImage,
) -> Result<(DynamicImage, u32, u32, u32, u32), Box<dyn Error>> {
    let guild_image_width = guild_image.width();
    let guild_image_height = guild_image.height();
    let image_width = image.width();
    let image_height = image.height();
    let x = (guild_image_width - image_width) / 2;
    let y = (guild_image_height - image_height) / 2;
    guild_image.copy_from(&image, x, y)?;
    Ok((
        guild_image.clone(),
        guild_image_width,
        guild_image_height,
        image_width,
        image_height,
    ))
}

pub async fn add_text_under_pseudo(
    guild_image: &mut DynamicImage,
    text: String,
    guild_image_width: u32,
    guild_image_height: u32,
    image_height: u32,
) -> Result<DynamicImage, Box<dyn Error>> {
    let renderer = TextRenderer::default();
    let text_png = renderer.render_text_to_png_data(text, 52, HEX_COLOR)?;
    let text_image = image::load_from_memory(&text_png.data)?;
    let text_image_width = text_image.width();
    let text_image_height = text_image.height();
    let x = (guild_image_width - text_image_width) / 2;
    let y = (guild_image_height - text_image_height) / 2 + image_height;
    guild_image.copy_from(&text_image, x, y)?;
    Ok(guild_image.clone())
}

pub async fn add_text_bottom_right(
    guild_image: &mut DynamicImage,
    text: String,
    guild_image_width: u32,
    guild_image_height: u32,
) -> Result<DynamicImage, Box<dyn Error>> {
    let renderer = TextRenderer::default();
    let text_png = renderer.render_text_to_png_data(text, 52, HEX_COLOR)?;
    let text_image = image::load_from_memory(&text_png.data)?;
    let text_image_width = text_image.width();
    let text_image_height = text_image.height();
    let x = (guild_image_width - text_image_width) + 20;
    let y = (guild_image_height - text_image_height) + 20;
    guild_image.copy_from(&text_image, x, y)?;
    Ok(guild_image.clone())
}
