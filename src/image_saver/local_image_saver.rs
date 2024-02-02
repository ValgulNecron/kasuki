use crate::error_enum::AppError;
use crate::error_enum::AppError::DifferedError;
use crate::error_enum::DifferedError::{FailedToCreateFolder, FailedToWriteFile};
use chrono::Local;
use std::fs;
use std::path::Path;

pub async fn local_image_save(
    guild_id: String,
    user_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<(), AppError> {
    let now = Local::now();
    let formatted = now.format("%m-%d-%Y_%H-%M").to_string();

    let file_path = format!("images/{}/{}", guild_id, user_id);
    // create the directory if it doesn't exist
    if !Path::new(&file_path).exists() {
        fs::create_dir_all(&file_path).map_err(|e| {
            DifferedError(FailedToCreateFolder(format!(
                "Failed to create directory. {}",
                e
            )))
        })?;
    }

    let filename = format!("{}_{}", filename, formatted);
    // write the file
    fs::write(format!("{}/{}", file_path, filename), image_data)
        .map_err(|e| DifferedError(FailedToWriteFile(format!("Failed to write image. {}", e))))?;

    Ok(())
}
