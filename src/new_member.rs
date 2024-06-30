use crate::constant::{NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, Member};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
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
    let guild_name = partial_guild.name.clone();
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
    let channel = match channel_id.to_channel(&ctx.http).await {
        Ok(channel) => channel,
        Err(e) => {
            error!("Failed to get the channel. {}", e);
            return;
        }
    };

    let image = if guild_specific.custom_image {
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
