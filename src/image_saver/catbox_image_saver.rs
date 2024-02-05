use crate::error_enum::AppError;
use crate::error_enum::AppError::DifferedError;
use crate::error_enum::DiffereCommanddError::FailedToUploadImage;
use std::env;
use tracing::debug;

pub async fn upload_image_catbox(filename: String) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => Some(a),
        Err(_) => None,
    };
    let url = catbox::file::from_file(filename, token)
        .await
        .map_err(|e| {
            DifferedError(FailedToUploadImage(format!(
                "Failed to upload image to catbox.moe. {}",
                e
            )))
        })?;
    debug!("Image uploaded to catbox.moe: {}", url);
    Ok(())
}
