use std::env;

use tracing::debug;

use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn upload_image_imgur(image_data: Vec<u8>) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => a,
        Err(e) => {
            return Err(AppError::new(
                format!("Failed to get the token for imgur.com. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            ));
        }
    };
    let upload_info = imgur::Handle::new(token).upload(&image_data).map_err(|e| {
        AppError::new(
            format!("Failed to upload image to imgur.com. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    debug!(
        "Image uploaded to imgur.com: {}",
        upload_info.link().unwrap()
    );
    Ok(())
}
