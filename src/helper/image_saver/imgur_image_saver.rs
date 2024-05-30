use imgurs::ImgurClient;
// Importing necessary libraries and modules
use std::{env, fs};
use tracing::debug;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `upload_image_imgur` is an asynchronous function that uploads an image to imgur.com.
/// It takes a `filename` and `image_data` as parameters.
/// `filename` is a String, and `image_data` is a Vec<u8>.
/// It returns a Result which is either an empty tuple or an `AppError`.
///
/// # Arguments
///
/// * `filename` - A string that represents the filename of the image.
/// * `image_data` - A Vec<u8> that represents the image data.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while getting the token, writing the image to a file, or uploading the image to imgur.com.
pub async fn upload_image_imgur(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
    // Get the token from the environment variables
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

    // Write the image data to a file
    fs::write(&filename, &image_data).map_err(|e| {
        AppError::new(
            format!("Failed to write image to file. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Create a new ImgurClient with the token
    let client = ImgurClient::new(token.as_str());

    // Upload the image to imgur.com
    let info = client.upload_image(filename.as_str()).await.map_err(|e| {
        AppError::new(
            format!("Failed to upload image to imgur.com. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Log the link of the uploaded image
    debug!("Image uploaded to imgur.com: {}", info.data.link);

    // Return Ok if the function executed successfully
    Ok(())
}
