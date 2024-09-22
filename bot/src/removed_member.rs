use crate::config::DbConfig;
use crate::custom_serenity_impl::InternalMemberAction::{BanAdd, Kick};
use crate::custom_serenity_impl::{InternalAction, InternalMemberAction};
use crate::new_member::{
    add_text, change_to_x256_url, encode_image, get_channel_id, get_image, get_server_image,
    load_guild_settings, overlay_image, send_image, XAlignment, YAlignment,
};
use crate::structure::message::removed_member::{load_localization_removed_member, RemovedMember};
use anyhow::anyhow;
use serenity::all::{AuditLogs, GuildId, User};
use serenity::client::Context as SerenityContext;
use tracing::{debug, info};

use anyhow::{Context, Result};
use chrono::Utc;

pub async fn removed_member_message(
    ctx: &SerenityContext,
    guild_id: GuildId,
    user: User,
    db_config: DbConfig,
) -> Result<()> {

    info!("Processing removed member message for guild: {}", guild_id);

    // Load guild settings
    let guild_settings = load_guild_settings(guild_id).await;

    debug!(?guild_settings, "Loaded guild settings");

    // Get channel ID
    let partial_guild = guild_id
        .to_partial_guild(&ctx.http)
        .await
        .context("Failed to get partial guild")?;

    let channel_id = get_channel_id(&guild_settings, &partial_guild)
        .ok_or_else(|| anyhow!("Failed to get the channel id"))?;

    debug!(?channel_id, "Obtained channel id");

    // Get server image
    let guild_image_data = get_server_image(guild_id.to_string(), &guild_settings)
        .ok_or_else(|| anyhow!("Failed to get the server image"))?;

    let mut guild_image = image::load_from_memory(&guild_image_data)
        .context("Failed to load guild image from memory")?;

    // Process user avatar
    let avatar_url = change_to_x256_url(user.face());

    debug!(?avatar_url, "Changed avatar URL to x64 size");

    let avatar_image = get_image(avatar_url)
        .await
        .context("Failed to download user avatar image")?;

    // Fetch audit logs
    let audit_log = guild_id
        .audit_logs(&ctx.http, None, None, None, Some(100))
        .await
        .context("Failed to fetch audit logs")?;

    // Load localization
    let local = load_localization_removed_member(guild_id.to_string(), db_config)
        .await
        .context("Failed to load localization for removed member")?;

    // Determine reason for removal
    let user_name = user.name.clone();

    let reason = determine_reason(&audit_log, &user, &local, &user_name);

    debug!(?reason, "Determined reason for removal");

    // Overlay user avatar on guild image
    let (mut guild_image, _, _, _, image_height) = overlay_image(&mut guild_image, avatar_image)
        .await
        .context("Failed to overlay user avatar on guild image")?;

    // Add reason text to image
    guild_image = add_text(
        &mut guild_image,
        reason,
        XAlignment::Center,
        YAlignment::Center,
        image_height,
    )
    .await
    .context("Failed to add reason text to image")?;

    // Add timestamp to image
    let now = Utc::now();

    let join_data = now.format("%m/%d/%Y %H:%M:%S").to_string();

    guild_image = add_text(
        &mut guild_image,
        join_data,
        XAlignment::Right,
        YAlignment::Bottom,
        0,
    )
    .await
    .context("Failed to add timestamp to image")?;

    // Encode final image
    let bytes = encode_image(guild_image).context("Failed to encode final image to bytes")?;

    // Send image to channel
    send_image(channel_id, bytes, &ctx.http)
        .await
        .context("Failed to send image to channel")?;

    info!(
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
