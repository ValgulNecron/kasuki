use std::{env, fs};

use imgurs::ImgurClient;
use tracing::debug;

use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn upload_image_imgur(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
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
    fs::write(&filename, &image_data).map_err(|e| {
        AppError::new(
            format!("Failed to write image to file. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    let client = ImgurClient::new(token.as_str());

    let info = client.upload_image(filename.as_str()).await.map_err(|e| {
        AppError::new(
            format!("Failed to upload image to imgur.com. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    debug!("Image uploaded to imgur.com: {}", info.data.link);
    Ok(())
}
