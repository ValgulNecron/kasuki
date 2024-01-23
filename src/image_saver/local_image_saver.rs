use crate::error_enum::AppError;
use std::fs;
use std::path::Path;

pub async fn local_image_save(
    guild_id: String,
    user_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<(), AppError> {
    let file_path = format!("images/{}/{}", guild_id, user_id);
    // create the directory if it doesn't exist
    if !Path::new(&file_path).exists() {
        fs::create_dir_all(&file_path).map_err(|_| {
            AppError::FailedToCreateFolder("Failed to create directory".to_string())
        })?;
    }

    // write the file
    fs::write(format!("{}/{}", file_path, filename), image_data)
        .map_err(|_| AppError::FailedToWriteFile("Failed to write image".to_string()))?;

    Ok(())
}
