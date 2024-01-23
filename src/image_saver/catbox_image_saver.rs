use crate::error_enum::AppError;
use std::env;
use tracing::debug;

pub async fn upload_image_catbox(filename: String) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => Some(a),
        Err(_) => None,
    };
    let url = catbox::file::from_file(filename, token)
        .await
        .map_err(|_| {
            AppError::FailedToUploadImage("Failed to upload image to catbox.moe".to_string())
        })?;
    debug!("Image uploaded to catbox.moe: {}", url);
    Ok(())
}
