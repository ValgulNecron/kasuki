use std::env;

use crate::error_management::error_enum::AppError;
use crate::image_saver::catbox_image_saver::upload_image_catbox;
use crate::image_saver::imgur_image_saver::upload_image_imgur;
use crate::image_saver::local_image_saver::local_image_save;

pub async fn image_saver(
    guild_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<(), AppError> {
    let saver_type = env::var("SAVE_IMAGE").unwrap_or("local".to_string());
    if saver_type == *"local" {
        local_image_save(guild_id, filename, image_data).await
    } else if saver_type == *"remote" {
        remote_saver(filename, image_data).await
    } else {
        Ok(())
    }
}

pub async fn remote_saver(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
    let saver_server = env::var("SAVE_SERVER").unwrap_or("catbox".to_string());
    if saver_server == *"catbox" {
        upload_image_catbox(filename, image_data).await
    } else if saver_server == *"imgur" {
        upload_image_imgur(filename, image_data).await
    } else {
        upload_image_catbox(filename, image_data).await
    }
}
