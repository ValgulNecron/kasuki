use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use std::{env, fs};
use tracing::debug;

pub async fn upload_image_catbox(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => Some(a),
        Err(_) => None,
    };
    fs::write(&filename, &image_data).map_err(|e| {
        AppError::new(
            format!("Failed to write image to file. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    let url = catbox::file::from_file(filename.clone(), token)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to upload image to catbox.moe. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;
    fs::remove_file(&filename)
        .map_err(|e|
            AppError::new(
                format!("Failed to remove file. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            ))?;
    debug!("Image uploaded to catbox.moe: {}", url);
    Ok(())
}
