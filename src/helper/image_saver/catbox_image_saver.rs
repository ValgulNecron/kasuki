use std::env;

// Importing necessary libraries and modules
use reqwest::multipart;
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
    let token = match env::var("TOKEN").map_err(|e| {
        AppError::new(
            format!("Failed to get the token. {}", e),
            ErrorType::Option,
            ErrorResponseType::Unknown,
        )
    }) {
        Ok(token) => token,
        Err(e) => return Err(e),
    };

    let form = multipart::Form::new()
        .text("reqtype", "fileupload")
        .text("userhash", token)
        .part(
            "fileToUpload",
            multipart::Part::stream(image_data).file_name(filename),
        );
    // Build the URL
    let url = "https://catbox.moe/user/api.php";

    // Send the request
    let client = reqwest::Client::new();
    let response = client.post(url).multipart(form).send().await.map_err(|e| {
        AppError::new(
            format!("Failed to upload image. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Unknown,
        )
    })?;

    debug!("Response status: {}", response.status());
    debug!("Response text: {:#?}", response.text().await);

    // Return Ok if the function executed successfully
    Ok(())
}
