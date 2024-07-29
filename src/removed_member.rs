use image::GenericImage;
use serenity::all::{Context, GuildId, User};
use text_to_png::TextRenderer;
use tracing::error;

use crate::constant::HEX_COLOR;
use crate::custom_serenity_impl::InternalAction;
use crate::custom_serenity_impl::InternalMemberAction::{BanAdd, Kick};
use crate::new_member::{
    get_channel_id, get_guild_image_bytes, get_server_image, load_guild_settings, send_member_image,
};
use crate::structure::message::removed_member::load_localization_removed_member;

pub async fn removed_member_message(ctx: &Context, guild_id: GuildId, user: User) {
    let ctx = ctx.clone();
    let user_name = user.name.clone();
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
    let avatar = user.face().replace("size=1024", "size=128");
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
        .audit_logs(&ctx.http, None, None, None, Some(100))
        .await
    {
        Ok(audit_log) => audit_log,
        Err(e) => {
            error!("Failed to get the audit log. {}", e);
            return;
        }
    };
    let reason;
    if audit_log.entries.is_empty() || audit_log.entries.is_empty() {
        reason = "User left".to_string()
    } else {
        let mut internal_action: InternalAction = audit_log.entries[0].action.into();
        for entry in &audit_log.entries {
            let target = match entry.target_id {
                Some(target) => target,
                None => continue,
            };
            if target.to_string() == user.id.to_string() {
                internal_action = entry.action.into();
                break;
            }
        }
        reason = if internal_action == InternalAction::Member(BanAdd) {
            let a = audit_log.entries[0].clone().reason.unwrap_or_default();
            if a.is_empty() {
                "User Banned".to_string()
            } else {
                format!("User Banned for {}", a)
            }
        } else if internal_action == InternalAction::Member(Kick) {
            let a = audit_log.entries[0].clone().reason.unwrap_or_default();
            if a.is_empty() {
                "User kicked".to_string()
            } else {
                format!("User kicked for {}", a)
            }
        } else {
            "User left".to_string()
        };
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
    let bytes = match get_guild_image_bytes(guild_image) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to get the bytes. {}", e);
            return;
        }
    };
    send_member_image(channel_id, bytes, &ctx.http).await;
}
