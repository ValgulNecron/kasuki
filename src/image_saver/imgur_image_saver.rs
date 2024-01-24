use crate::error_enum::AppError;
use crate::error_enum::AppError::FailedToUploadImage;
use std::env;
use tracing::debug;

pub async fn upload_image_imgur(image_data: Vec<u8>) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => a,
        Err(_) => {
            return Err(FailedToUploadImage(
                "Failed to get the token for imgur.com".to_string(),
            ))
        }
    };
    let upload_info = imgur::Handle::new(token)
        .upload(&image_data)
        .map_err(|_| FailedToUploadImage("Failed to upload image to imgur.com".to_string()))?;
    debug!(
        "Image uploaded to imgur.com: {}",
        upload_info.link().unwrap()
    );
    Ok(())
}
