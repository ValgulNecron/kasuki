use crate::config::BotConfigDetails;
use crate::constant::HEX_COLOR;
use crate::custom_serenity_impl::{InternalAction, InternalMemberAction};
use crate::custom_serenity_impl::InternalMemberAction::{BanAdd, Kick};
use crate::new_member::{
    encode_image, get_channel_id, get_server_image, load_guild_settings, send_image,
};
use crate::structure::message::removed_member::load_localization_removed_member;
use image::GenericImage;
use serenity::all::{GuildId, User};
use serenity::client::Context;
use std::any::Any;
use text_to_png::TextRenderer;
use tracing::{error, trace};

pub async fn removed_member_message(
    ctx: &Context,
    guild_id: GuildId,
    user: User,
    db_config: BotConfigDetails,
) {
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
    trace!(?guild_settings);

    let channel_id = if let Some(channel_id) = get_channel_id(&guild_settings, &partial_guild) {
        channel_id
    } else {
        error!("Failed to get the channel id.");
        return;
    };
    trace!(?channel_id);

    let guild_image = if let Some(img) = get_server_image(guild_id.to_string(), &guild_settings) {
        img
    } else {
        error!("Failed to get the server image.");
        return;
    };

    let mut guild_image = match image::load_from_memory(&guild_image) {
        Ok(image) => image,
        Err(e) => {
            error!("Failed to load the image. {}", e);
            return;
        }
    };

    let avatar = user
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

    let local = match load_localization_removed_member(guild_id.to_string(), db_config).await {
        Ok(local) => local,
        Err(e) => {
            error!("Failed to load the localization. {}", e);
            return;
        }
    };
    let reason = match audit_log.entries.is_empty() {
        true => local.bye.replace("$user$", &user_name),
        false => {
            let mut text: String = String::new();
            for entry in &audit_log.entries {
                let target = match entry.target_id {
                    Some(target) => target,
                    None => continue,
                };
                if target.to_string() == user.id.to_string() {
                    let reason: Option<String> = entry.reason.clone();
                    let action: InternalMemberAction = InternalAction::from(entry.action).into();
                    match (action, reason) {
                        (BanAdd, Some(reason)) => {
                            text = local
                                .ban_for
                                .replace("$user$", &user_name)
                                .replace("$reason$", &reason)
                        }
                        (Kick, Some(reason)) => {
                            text = local
                                .kick_for
                                .replace("$user$", &user_name)
                                .replace("$reason$", &reason)
                        }
                        (BanAdd, None) => text = local.ban.replace("$user$", &user_name),
                        (Kick, None) => text = local.kick.replace("$user$", &user_name),
                        _ => text = local.bye.replace("$user$", &user_name),
                    }
                    break;
                }
            }
            text
        }
    };
    trace!(reason);

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
    let text_png = match renderer.render_text_to_png_data(reason, 52, HEX_COLOR) {
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
