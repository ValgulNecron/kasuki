use std::error::Error;

// Importing necessary libraries and modules
use std::fs;
use std::path::Path;

use chrono::Local;

/// `local_image_save` is an asynchronous function that saves an image locally.
/// It takes a `guild_id`, `filename`, and `image_data` as parameters.
/// `guild_id` and `filename` are both Strings, and `image_data` is a Vec<u8>.
/// It returns a Result which is either an empty tuple or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string that represents the guild id.
/// * `filename` - A string that represents the filename of the image.
/// * `image_data` - A Vec<u8> that represents the image data.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while creating the directory or writing the file.

pub async fn local_image_save(
    guild_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<(), Box<dyn Error>> {

    // Get the current date and time
    let now = Local::now();

    // Format the date and time
    let formatted = now.format("%m-%d-%Y_%H-%M").to_string();

    // Define the file path
    let file_path = format!("images/{}/", guild_id);

    // Check if the directory exists, if not, create it
    if !Path::new(&file_path).exists() {

        fs::create_dir_all(&file_path)?;
    }

    // Format the filename
    let filename = format!("{}_{}", formatted, filename);

    // Write the image data to the file
    fs::write(format!("{}/{}", file_path, filename), image_data)?;

    // Return Ok if the function executed successfully
    Ok(())
}
