use std::fs;
use std::path::Path;

use chrono::Local;

use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn local_image_save(
    guild_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<(), AppError> {
    let now = Local::now();
    let formatted = now.format("%m-%d-%Y_%H-%M").to_string();

    let file_path = format!("images/{}/", guild_id);
    // create the directory if it doesn't exist
    if !Path::new(&file_path).exists() {
        fs::create_dir_all(&file_path).map_err(|e| {
            AppError::new(
                format!("Failed to create directory. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;
    }

    let filename = format!("{}_{}", formatted, filename);
    // write the file
    fs::write(format!("{}/{}", file_path, filename), image_data).map_err(|e| {
        AppError::new(
            format!("Failed to write image. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    Ok(())
}
