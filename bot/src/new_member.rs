use crate::config::BotConfigDetails;
use crate::constant::{HEX_COLOR, NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::structure::message::new_member::load_localization_new_member;
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
use tracing::{error, trace};

pub async fn new_member_message(ctx: &Context, member: &Member, db_config: BotConfigDetails) {
    let ctx = ctx.clone();
    let user_name = member.user.name.clone();
    let guild_id = member.guild_id;
    let partial_guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(guild) => guild,
        Err(e) => {
            error!("Failed to get the guild. {}", e);
            return;
        }
    };
    let guild_settings = load_guild_settings(guild_id).await;
    trace!(?guild_settings);

    let channel_id = if let Some(channel_id) = get_channel_id(&guild_settings, &partial_guild) {
        channel_id
    } else {
        return;
    };
    trace!(?channel_id);

    let guild_image = if let Some(img) = get_server_image(guild_id.to_string(), &guild_settings) {
        img
    } else {
        return;
    };

    let mut guild_image = match image::load_from_memory(&guild_image) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };

    let avatar = member
        .face()
        .replace("?size=4096", "?size=64")
        .replace("?size=2048", "?size=64")
        .replace("?size=1024", "?size=64")
        .replace("?size=512", "?size=64")
        .replace("?size=256", "?size=64")
        .replace("?size=128", "?size=64");
    trace!(?avatar);

    let client = reqwest::Client::new();
    let res = match client.get(avatar).send().await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to get the image. {}", e);
            return;
        }
    };
    let body = match res.bytes().await {
        Ok(body) => body,
        Err(e) => {
            error!("Failed to get the image body. {}", e);
            return;
        }
    };
    let image = match image::load_from_memory(&body) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };

    let guild_image_width = guild_image.width();
    let guild_image_height = guild_image.height();
    let image_width = image.width();
    let image_height = image.height();
    // overlay the image on the guild image at the center
    let x = (guild_image_width - image_width) / 2;
    let y = (guild_image_height - image_height) / 2;
    match guild_image.copy_from(&image, x, y) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to overlay the image. {}", e);
        }
    }
    let local = match load_localization_new_member(guild_id.to_string(), db_config).await {
        Ok(local) => local.welcome,
        Err(_) => "Welcome $user$".to_string(),
    };
    let text = local.replace("$user$", &user_name);
    let renderer = TextRenderer::default();
    let text_png = match renderer.render_text_to_png_data(text, 52, HEX_COLOR) {
        Ok(text_png) => text_png,
        Err(e) => {
            error!("Failed to render the text to png. {}", e);
            return;
        }
    };
    let text_image = match image::load_from_memory(&text_png.data) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };
    let text_image_width = text_image.width();
    let text_image_height = text_image.height();
    let x = (guild_image_width - text_image_width) / 2;
    // under the user pfp
    let y = (guild_image_height - text_image_height) / 2 + image_height;
    match guild_image.copy_from(&text_image, x, y) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to overlay the image. {}", e);
        }
    }
    let join_data = member.joined_at.unwrap_or_default();
    // timestamp in format mm/dd/yyyy hh:mm:ss
    let join_data = join_data.format("%m/%d/%Y %H:%M:%S").to_string();
    let renderer = TextRenderer::default();
    let text_png = match renderer.render_text_to_png_data(join_data, 52, HEX_COLOR) {
        Ok(text_png) => text_png,
        Err(e) => {
            error!("Failed to render the text to png. {}", e);
            return;
        }
    };
    let text_image = match image::load_from_memory(&text_png.data) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };
    let text_image_width = text_image.width();
    let text_image_height = text_image.height();
    // bottom right corner
    let x = guild_image_width - text_image_width - (guild_image_width / 100);
    let y = guild_image_height - text_image_height;
    match guild_image.copy_from(&text_image, x, y) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to overlay the image. {}", e);
        }
    }
    let bytes = match encode_image(guild_image) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to get the bytes. {}", e);
            return;
        }
    };
    match send_image(channel_id, bytes, &ctx.http).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to send the image. {}", e);
        }
    };
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
