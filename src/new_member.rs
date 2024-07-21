use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Cursor;

use image::ImageFormat::WebP;
use image::{DynamicImage, GenericImage};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, Member, PartialGuild};
use serenity::builder::CreateAttachment;
use text_to_png::TextRenderer;
use tracing::{error, trace};

use crate::constant::{HEX_COLOR, NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::helper::error_management::error_enum;
use crate::structure::message::new_member::load_localization_new_member;

pub async fn new_member_message(ctx: &Context, member: &Member) {
    trace!("New member message.");
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
    let channel_id = if let Some(channel_id) = get_channel_id(&guild_settings, &partial_guild) {
        channel_id
    } else {
        return;
    };

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
    let avatar = member.user.face().replace("size=1024", "size=128");
    trace!(avatar);
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
    let local = match load_localization_new_member(guild_id.to_string(), "db".to_string()).await {
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

    let rgba8_image = guild_image.to_rgba8();
    let mut bytes: Vec<u8> = Vec::new();
    match rgba8_image.write_to(&mut Cursor::new(&mut bytes), WebP) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to write the image to the buffer. {}", e);
            return;
        }
    };
    let attachement = CreateAttachment::bytes(bytes, "new_member.webp");
    let builder = CreateMessage::new().add_file(attachement);
    match channel_id.send_message(&ctx.http, builder).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to send the message. {}", e);
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
    let hashmap: HashMap<String, NewMemberSetting> =
        serde_json::from_str(&content).unwrap_or_default();
    hashmap
        .get(&guild_id.to_string())
        .unwrap_or(&NewMemberSetting::default())
        .clone()
}

pub fn load_new_member_image(guild_id: String) -> Option<Vec<u8>> {
    // load the image from the file
    let path = format!("{}{}.png", NEW_MEMBER_IMAGE_PATH, guild_id);
    let image = fs::read(path);
    match image {
        Ok(image) => Some(image),
        Err(e) => {
            error!("Failed to load the image. {}", e);
            None
        }
    }
}

pub fn create_default_new_member_image() -> Result<Vec<u8>, Box<dyn Error>> {
    let width = 2000;
    let height = width / 4;
    let img = DynamicImage::new_rgba8(width, height);
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), WebP)
        .map_err(|e| error_enum::Error::ImageProcessing(format!("{:#?}", e)))?;
    Ok(bytes)
}

pub fn get_server_image(guild_id: String, guild_settings: &NewMemberSetting) -> Option<Vec<u8>> {
    let img = if guild_settings.custom_image {
        let image = load_new_member_image(guild_id.to_string());
        match image {
            Some(image) => image,
            None => {
                error!("Failed to load the image.");
                return None;
            }
        }
    } else {
        let image = create_default_new_member_image();
        match image {
            Ok(image) => image,
            Err(e) => {
                error!("Failed to create the default image. {}", e);
                return None;
            }
        }
    };
    Some(img)
}

pub fn get_channel_id(
    guild_settings: &NewMemberSetting,
    partial_guild: &PartialGuild,
) -> Option<ChannelId> {
    let channel_id = if guild_settings.custom_channel {
        ChannelId::from(guild_settings.channel_id)
    } else {
        match partial_guild.system_channel_id {
            Some(channel_id) => channel_id,
            None => {
                error!("Failed to get the system channel id.");
                return None;
            }
        }
    };
    Some(channel_id)
}
