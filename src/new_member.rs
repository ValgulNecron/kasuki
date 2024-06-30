use crate::constant::{HEX_COLOR, NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use image::ImageFormat::WebP;
use image::{DynamicImage, GenericImage, ImageFormat};
use serde::{Deserialize, Serialize};
use serenity::all::{Attachment, ChannelId, Context, CreateMessage, Member};
use serenity::builder::CreateAttachment;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use text_to_png::TextRenderer;
use tracing::error;

pub async fn new_member_message(ctx: &Context, member: &Member) {
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
    let content = fs::read_to_string(NEW_MEMBER_PATH).unwrap_or_else(|_| String::new());
    let hashmap: HashMap<String, NewMemberSetting> =
        serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new());
    let guild_specific = hashmap
        .get(&guild_id.to_string())
        .unwrap_or(&NewMemberSetting {
            custom_channel: false,
            channel_id: 0,
            custom_image: false,
            show_username: true,
            show_time_join: true,
        });
    let channel_id = if guild_specific.custom_channel {
        ChannelId::from(guild_specific.channel_id)
    } else {
        match partial_guild.system_channel_id {
            Some(channel_id) => channel_id,
            None => return,
        }
    };

    let guild_image = if guild_specific.custom_image {
        let image = load_new_member_image(guild_id.to_string());
        match image {
            Some(image) => image,
            None => return,
        }
    } else {
        let image = create_default_new_member_image();
        match image {
            Ok(image) => image,
            Err(_) => return,
        }
    };
    let mut guild_image = match image::load_from_memory(&guild_image) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };
    let avatar = member.user.face();
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

    let renderer = TextRenderer::default();
    let text_png = match renderer.render_text_to_png_data(user_name, 52, HEX_COLOR) {
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
    let y = (guild_image_height - text_image_height) / 2 - (image_height / 2 + image_height / 16);
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

pub fn create_default_new_member_image() -> Result<Vec<u8>, AppError> {
    let width = 2000;
    let height = width / 4;
    let img = DynamicImage::new_rgba16(width, height);
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)
        .map_err(|e| {
            AppError::new(
                format!("Failed to write the image to the buffer. {}", e),
                ErrorType::Image,
                ErrorResponseType::None,
            )
        })?;
    Ok(bytes)
}
