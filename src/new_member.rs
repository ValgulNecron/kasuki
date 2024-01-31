use crate::error_enum::AppError;
use crate::error_enum::AppError::FailedToCreateDirectory;
use serenity::all::{Context, Member};
use std::fs;
use std::path::Path;

pub async fn new_member(ctx: Context, member: &mut Member) -> Result<(), AppError> {
    let path = "server_image";

    if !Path::new(path).exists() {
        fs::create_dir_all(path).map_err(|e| {
            FailedToCreateDirectory(format!("Failed to create the directory {}", e))
        })?;
    }

    let full_image_path = format!("{}/{}", path, member.guild_id.to_string()).as_str();

    let channel_id = get_channel_to_send().await?;

    if Path::new(full_image_path).exists() {
    } else {
    }

    Ok(())
}

async fn get_channel_to_send() -> Result<(), AppError> {
    Ok(())
}
