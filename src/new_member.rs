use crate::error_enum::AppError;
use crate::error_enum::AppError::JoiningError;
use crate::error_enum::JoiningError::FailedToCreateDirectory;
use serenity::all::{Context, Member};
use std::fs;
use std::path::Path;
use crate::constant::SERVER_IMAGE_PATH;

pub async fn new_member(ctx: Context, member: &mut Member) -> Result<(), AppError> {
    if !Path::new(SERVER_IMAGE_PATH).exists() {
        fs::create_dir_all(SERVER_IMAGE_PATH).map_err(|e| {
            JoiningError(FailedToCreateDirectory(format!(
                "Failed to create the directory {}",
                e
            )))
        })?;
    }

    let fip = format!("{}/{}", SERVER_IMAGE_PATH, member.guild_id);
    let full_image_path = fip.as_str();

    let channel_id = get_channel_to_send().await?;

    if Path::new(full_image_path).exists() {
    } else {
    }

    Ok(())
}

async fn get_channel_to_send() -> Result<(), AppError> {
    Ok(())
}
