use crate::error_enum::AppError;
use crate::error_enum::AppError::DifferedError;
use crate::error_enum::DifferedError::FailedToUploadImage;
use std::env;
use tracing::debug;

pub async fn upload_image_imgur(image_data: Vec<u8>) -> Result<(), AppError> {
    let token = match env::var("TOKEN") {
        Ok(a) => a,
        Err(e) => {
            return Err(DifferedError(FailedToUploadImage(format!(
                "Failed to get the token for imgur.com. {}",
                e
            ))));
        }
    };
    let upload_info = imgur::Handle::new(token).upload(&image_data).map_err(|e| {
        DifferedError(FailedToUploadImage(format!(
            "Failed to upload image to imgur.com. {}",
            e
        )))
    })?;
    debug!(
        "Image uploaded to imgur.com: {}",
        upload_info.link().unwrap()
    );
    Ok(())
}
