use anyhow::Result;

// Importing necessary libraries and modules
use std::fs;
use std::path::Path;

use chrono::Local;

pub async fn local_image_save(
    guild_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<()> {
    let now = Local::now();

    let formatted = now.format("%m-%d-%Y_%H-%M").to_string();

    let file_path = format!("images/{}/", guild_id);

    if !Path::new(&file_path).exists() {
        fs::create_dir_all(&file_path)?;
    }

    let filename = format!("{}_{}", formatted, filename);

    fs::write(format!("{}/{}", file_path, filename), image_data)?;

    Ok(())
}
