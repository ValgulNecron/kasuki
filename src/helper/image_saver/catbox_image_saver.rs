// Importing necessary libraries and modules
use std::{env, fs};

use tracing::debug;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// `upload_image_catbox` is an asynchronous function that uploads an image to catbox.moe.
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
/// This function will return an `AppError` if it encounters any issues while getting the token, writing the image to a file, or uploading the image to catbox.moe.
pub async fn upload_image_catbox(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
    // Get the token from the environment variables
    let token = match env::var("TOKEN") {
        Ok(a) => Some(a),
        Err(_) => None,
    };

    // Write the image data to a file
    fs::write(&filename, &image_data).map_err(|e| {
        AppError::new(
            format!("Failed to write image to file. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Upload the image to catbox.moe
    let url = catbox::file::from_file(filename.clone(), token)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to upload image to catbox.moe. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    // Remove the local file after upload
    fs::remove_file(&filename).map_err(|e| {
        AppError::new(
            format!("Failed to remove file. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Log the URL of the uploaded image
    debug!("Image uploaded to catbox.moe: {}", url);

    // Return Ok if the function executed successfully
    Ok(())
}
