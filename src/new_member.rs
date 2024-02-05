use crate::constant::SERVER_IMAGE_PATH;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{DifferedError, JoiningError, NewMemberError};
use crate::error_enum::JoiningError::FailedToCreateDirectory;
use crate::error_enum::NewMemberError::NewMemberErrorOptionError;
use serenity::all::{ChannelId, Context, CreateAttachment, CreateMessage, Member, PartialGuild};
use std::fs;
use std::path::Path;
use crate::error_enum::DifferedCommandError::DifferedCommandSendingError;

pub async fn new_member(ctx: Context, member: &mut Member) -> Result<(), AppError> {
    if !Path::new(SERVER_IMAGE_PATH).exists() {
        fs::create_dir_all(SERVER_IMAGE_PATH).map_err(|e| {
            JoiningError(FailedToCreateDirectory(format!(
                "Failed to create the directory {}",
                e
            )))
        })?;
    }

    let fip = format!("{}/{}.webp", SERVER_IMAGE_PATH, member.guild_id);
    let full_image_path = fip.as_str();

    if Path::new(full_image_path).exists() {
        let guild = member
            .guild_id
            .to_partial_guild_with_counts(&ctx.http)
            .await
            .map_err(|e| {
                NewMemberError(NewMemberErrorOptionError(format!(
                    "there was an error getting the guild. {}",
                    e
                )))
            })?;
        let _channel = get_channel_to_send(guild).await?;
    } else {
        let fip = format!("{}/default.webp", SERVER_IMAGE_PATH);
        let full_image_path = fip.as_str();
        let guild = member
            .guild_id
            .to_partial_guild_with_counts(&ctx.http)
            .await
            .map_err(|e| {
                NewMemberError(NewMemberErrorOptionError(format!(
                    "there was an error getting the guild. {}",
                    e
                )))
            })?;
        let channel = get_channel_to_send(guild).await?;
        let attachment = CreateAttachment::path(full_image_path).await.map_err(|e| {
            NewMemberError(NewMemberErrorOptionError(format!(
                "there was an error sending the image. {}",
                e
            )))
        })?;
        let mut create_message = CreateMessage::default();
        create_message = create_message.content("This is a test");
        create_message = create_message.add_file(attachment);
        channel.send_message(&ctx.http, create_message).await.map_err(|e| {
            NewMemberError(NewMemberErrorOptionError(format!(
                "there was an error sending the message. {}",
                e
            )))
        })?;
    }

    Ok(())
}

async fn get_channel_to_send(guild: PartialGuild) -> Result<ChannelId, AppError> {
    guild
        .system_channel_id
        .ok_or(NewMemberError(NewMemberErrorOptionError(
            "there was an error getting the channel to send".to_string(),
        )))
}
