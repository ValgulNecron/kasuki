use crate::constant::{HEX_COLOR, NEW_MEMBER_PATH};
use crate::new_member::{create_default_new_member_image, load_new_member_image, NewMemberSetting};
use crate::structure::message::removed_member::load_localization_removed_member;
use image::ImageFormat::WebP;
use image::{GenericImage};
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, User};
use serenity::builder::CreateAttachment;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use text_to_png::TextRenderer;
use tracing::{error, trace};

pub async fn removed_member_message(ctx: &Context, guild_id: GuildId, user: User) {
    trace!("New member message.");
    let ctx = ctx.clone();
    let user_name = user.name.clone();
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
            None => {
                error!("Failed to get the system channel id.");
                return;
            }
        }
    };

    let guild_image = if guild_specific.custom_image {
        let image = load_new_member_image(guild_id.to_string());
        match image {
            Some(image) => image,
            None => {
                error!("Failed to load the image.");
                return;
            }
        }
    } else {
        let image = create_default_new_member_image();
        match image {
            Ok(image) => image,
            Err(e) => {
                error!("Failed to create the default image. {}", e);
                return;
            }
        }
    };
    let mut guild_image = match image::load_from_memory(&guild_image) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };
    let avatar = user.face().replace("size=1024", "size=128");
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

    // get server audit log
    let audit_log = match guild_id
        .audit_logs(&ctx.http, None, Some(user.id), None, Some(1))
        .await
    {
        Ok(audit_log) => audit_log,
        Err(e) => {
            error!("Failed to get the audit log. {}", e);
            return;
        }
    };
    let mut reason = if audit_log.entries.is_empty() {
        "User left".to_string()
    } else {
        "".to_string()
    };
    let internal_action: InternalAction = audit_log.entries[0].action.into();
    reason = if internal_action == InternalAction::Member(BanAdd) {
        let a = audit_log.entries[0].clone().reason.unwrap_or_default();
        format!("User Banned for {}", a)
    } else if internal_action == InternalAction::Member(Kick) {
        let a = audit_log.entries[0].clone().reason.unwrap_or_default();
        format!("User kicked for {}", a)
    } else {
        "User left".to_string()
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
    let local = match load_localization_removed_member(guild_id.to_string(), "db".to_string()).await
    {
        Ok(local) => local.bye,
        Err(_) => "$user$ quited the server".to_string(),
    };
    let text = local
        .replace("$user$", &user_name)
        .replace("$reason$", &reason);
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
    let now = chrono::Utc::now();
    let join_data = now.format("%m/%d/%Y %H:%M:%S").to_string();
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

use crate::custom_serenity_impl::InternalAction;
use crate::custom_serenity_impl::InternalMemberAction::{BanAdd, Kick};
