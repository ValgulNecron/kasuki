use crate::constant::SERVER_IMAGE_PATH;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{JoiningError, NewMemberError};
use crate::error_enum::JoiningError::FailedToCreateDirectory;
use crate::error_enum::NewMemberError::NewMemberErrorOptionError;
use serenity::all::{ChannelId, Context, Member, PartialGuild};
use std::fs;
use std::path::Path;

pub async fn new_member(ctx: Context, member: &mut Member) -> Result<(), AppError> {
    if !Path::new(SERVER_IMAGE_PATH).exists() {
        fs::create_dir_all(SERVER_IMAGE_PATH).map_err(|e| {
            JoiningError(FailedToCreateDirectory(format!(
                "Failed to create the directory {}",
                e
            )))
        })?;
    }

    let fip = format!("{}/{}.png", SERVER_IMAGE_PATH, member.guild_id);
    let full_image_path = fip.as_str();

    if Path::new(full_image_path).exists() {
        let guild = member
            .guild_id
            .to_partial_guild_with_counts(ctx.http)
            .await
            .map_err(|e| {
                NewMemberError(NewMemberErrorOptionError(format!(
                    "there was an error getting the guild. {}",
                    e
                )))
            })?;
        let _channel = get_channel_to_send(guild).await?;
    } else {
        let fip = format!("{}/default.png", SERVER_IMAGE_PATH);
        let _full_image_path = fip.as_str();
    }

    Ok(())
}

async fn get_channel_to_send(guild: PartialGuild) -> Result<ChannelId, AppError> {
    let channel_id = guild
        .system_channel_id
        .ok_or(NewMemberError(NewMemberErrorOptionError(
            "there was an error getting the channel to send".to_string(),
        )));
    if let Err(ref _e) = channel_id {
        Ok(channel_id.unwrap_or_default())
    } else {
        channel_id
    }
}
