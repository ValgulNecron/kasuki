use crate::error_enum::AppError;
use crate::error_enum::AppError::DifferedError;
use crate::error_enum::DifferedCommandError::FailedToUploadImage;
use std::{env, fs};
use tracing::debug;

pub async fn upload_image_catbox(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => Some(a),
        Err(_) => None,
    };
    fs::write(&filename, &image_data).map_err(|e| {
        DifferedError(FailedToUploadImage(format!(
            "Failed to write image to file. {}",
            e
        )))
    })?;
    let url = catbox::file::from_file(filename.clone(), token)
        .await
        .map_err(|e| {
            DifferedError(FailedToUploadImage(format!(
                "Failed to upload image to catbox.moe. {}",
                e
            )))
        })?;
    fs::remove_file(&filename)
        .map_err(|e| DifferedError(FailedToUploadImage(format!("Failed to remove file. {}", e))))?;
    debug!("Image uploaded to catbox.moe: {}", url);
    Ok(())
}
