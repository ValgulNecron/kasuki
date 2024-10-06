use std::error::Error;
use std::io::{Seek, Write};
use tracing::debug;

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

pub async fn upload_image_catbox(
    filename: String,
    image_data: Vec<u8>,
    token: String,
) -> Result<(), Box<dyn Error>> {
    let suffix = filename.split(".").last().unwrap_or_default().to_string();

    let mut file = tempfile::Builder::new()
        .suffix(&format!(".{}", suffix))
        .tempfile()?;

    file.write_all(&image_data)?;

    file.seek(std::io::SeekFrom::Start(0))?;

    debug!("File path: {}", file.path().to_str().unwrap());

    catbox::file::from_file(file.path().to_str().unwrap(), Some(&token)).await?;

    // Return Ok if the function executed successfully
    Ok(())
}
