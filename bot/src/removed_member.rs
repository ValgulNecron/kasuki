use crate::config::DbConfig;
use crate::custom_serenity_impl::InternalMemberAction::{BanAdd, Kick};
use crate::custom_serenity_impl::{InternalAction, InternalMemberAction};
use crate::new_member::{
    add_text, change_to_x64_url, encode_image, get_channel_id, get_image, get_server_image,
    load_guild_settings, overlay_image, send_image, XAlignment, YAlignment,
};
use crate::structure::message::removed_member::{load_localization_removed_member, RemovedMember};
use anyhow::anyhow;
use serenity::all::{AuditLogs, GuildId, User};
use serenity::client::Context;
use std::error::Error;
use tracing::trace;

pub async fn removed_member_message(
    ctx: &Context,
    guild_id: GuildId,
    user: User,
    db_config: DbConfig,
) -> Result<(), Box<dyn Error>> {

    trace!("Starting removed_member_message function");

    let ctx = ctx.clone();

    let user_name = user.name.clone();

    let partial_guild = guild_id.to_partial_guild(&ctx.http).await?;

    let guild_settings = load_guild_settings(guild_id).await;

    trace!(?guild_settings, "Loaded guild settings");

    let channel_id = get_channel_id(&guild_settings, &partial_guild)
        .ok_or(anyhow!("Failed to get the channel id."))?;

    trace!(?channel_id, "Obtained channel id");

    let guild_image = get_server_image(guild_id.to_string(), &guild_settings)
        .ok_or(anyhow!("Failed to get the server image."))?;

    trace!("Obtained server image");

    let mut guild_image = image::load_from_memory(&guild_image)?;

    trace!("Loaded guild image from memory");

    let avatar = change_to_x64_url(user.face());

    trace!(?avatar, "Changed avatar URL to x64 size");

    let image = get_image(avatar).await?;

    trace!("Downloaded user avatar image");

    let audit_log = guild_id
        .audit_logs(&ctx.http, None, None, None, Some(100))
        .await?;

    trace!("Fetched audit logs");

    let local = load_localization_removed_member(guild_id.to_string(), db_config).await?;

    trace!("Loaded localization for removed member");

    let reason = determine_reason(&audit_log, &user, &local, &user_name);

    trace!(?reason, "Determined reason for removal");

    let (mut guild_image, _, _, _, image_height) = overlay_image(&mut guild_image, image).await?;

    trace!("Overlayed user avatar on guild image");

    guild_image = add_text(
        &mut guild_image,
        reason,
        XAlignment::Center,
        YAlignment::Center,
        image_height,
    )
    .await?;

    trace!("Added reason text to image");

    let now = chrono::Utc::now();

    let join_data = now.format("%m/%d/%Y %H:%M:%S").to_string();

    guild_image = add_text(
        &mut guild_image,
        join_data,
        XAlignment::Right,
        YAlignment::Bottom,
        0,
    )
    .await?;

    trace!("Added timestamp to image");

    let bytes = encode_image(guild_image)?;

    trace!("Encoded final image to bytes");

    send_image(channel_id, bytes, &ctx.http).await?;

    trace!("Sent image to channel");

    trace!(
        "Successfully sent removed member message for user: {}",
        user_name
    );

    Ok(())
}

fn determine_reason(
    audit_log: &AuditLogs,
    user: &User,
    local: &RemovedMember,
    user_name: &str,
) -> String {

    if audit_log.entries.is_empty() {

        return local.bye.replace("$user$", user_name);
    }

    for entry in &audit_log.entries {

        if let Some(target) = entry.target_id {

            if target.to_string() == user.id.to_string() {

                let reason = entry.reason.clone();

                let action: InternalMemberAction = InternalAction::from(entry.action).into();

                return match (action, reason) {
                    (BanAdd, Some(reason)) => local
                        .ban_for
                        .replace("$user$", user_name)
                        .replace("$reason$", &reason),
                    (Kick, Some(reason)) => local
                        .kick_for
                        .replace("$user$", user_name)
                        .replace("$reason$", &reason),
                    (BanAdd, None) => local.ban.replace("$user$", user_name),
                    (Kick, None) => local.kick.replace("$user$", user_name),
                    _ => local.bye.replace("$user$", user_name),
                };
            }
        }
    }

    local.bye.replace("$user$", user_name)
}
