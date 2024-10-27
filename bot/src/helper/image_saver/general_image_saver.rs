use anyhow::Result;

// Importing necessary libraries and modules
use crate::helper::image_saver::catbox_image_saver::upload_image_catbox;
use crate::helper::image_saver::local_image_saver::local_image_save;

pub async fn image_saver(
    guild_id: String,
    filename: String,
    image_data: Vec<u8>,
    saver_server: String,
    token: String,
    save_type: String,
) -> Result<()> {
    if save_type == *"local" {
        local_image_save(guild_id, filename, image_data).await
    } else if save_type == *"remote" {
        remote_saver(filename, image_data, saver_server, token).await
    } else {
        Ok(())
    }
}

pub async fn remote_saver(
    filename: String,
    image_data: Vec<u8>,
    saver_server: String,
    token: String,
) -> Result<()> {
    match saver_server.as_str() {
        _ => upload_image_catbox(filename, image_data, token).await,
    }
}
