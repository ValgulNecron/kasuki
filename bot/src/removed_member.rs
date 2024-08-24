use crate::config::BotConfigDetails;
use crate::constant::HEX_COLOR;
use crate::custom_serenity_impl::InternalMemberAction::{BanAdd, Kick};
use crate::custom_serenity_impl::{InternalAction, InternalMemberAction};
use crate::new_member::{
    add_text_bottom_right, add_text_under_pseudo, change_to_x64_url, encode_image, get_channel_id,
    get_image, get_server_image, load_guild_settings, overlay_image, send_image,
};
use crate::structure::message::removed_member::load_localization_removed_member;
use anyhow::anyhow;
use image::GenericImage;
use serenity::all::{GuildId, User};
use serenity::client::Context;
use std::error::Error;
use text_to_png::TextRenderer;
use tracing::{error, trace};

pub async fn removed_member_message(
    ctx: &Context,
    guild_id: GuildId,
    user: User,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let ctx = ctx.clone();
    let user_name = user.name.clone();
    let partial_guild = guild_id.to_partial_guild(&ctx.http).await?;
    let guild_settings = load_guild_settings(guild_id).await;
    trace!(?guild_settings);

    let channel_id = get_channel_id(&guild_settings, &partial_guild)
        .ok_or(anyhow!("Failed to get the channel id."))?;
    trace!(?channel_id);

    let guild_image = get_server_image(guild_id.to_string(), &guild_settings)
        .ok_or(anyhow!("Failed to get the server image."))?;

    let mut guild_image = image::load_from_memory(&guild_image)?;

    let avatar = change_to_x64_url(user.face());
    trace!(?avatar);

    let image = get_image(avatar).await?;

    let audit_log = guild_id
        .audit_logs(&ctx.http, None, None, None, Some(100))
        .await?;

    let local = load_localization_removed_member(guild_id.to_string(), db_config).await?;
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

    let (mut guild_image, guild_image_width, guild_image_height, _, image_height) =
        overlay_image(&mut guild_image, image).await?;
    guild_image = add_text_under_pseudo(
        &mut guild_image,
        reason,
        guild_image_width,
        guild_image_height,
        image_height,
    )
    .await?;
    let now = chrono::Utc::now();
    let join_data = now.format("%m/%d/%Y %H:%M:%S").to_string();
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
