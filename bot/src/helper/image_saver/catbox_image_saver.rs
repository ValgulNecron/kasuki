use anyhow::{anyhow, Result};
use std::io::{Seek, Write};
use tracing::debug;

pub async fn upload_image_catbox(
    filename: String,
    image_data: Vec<u8>,
    token: String,
) -> Result<()> {
    let suffix = filename.split(".").last().unwrap_or_default().to_string();

    let mut file = tempfile::Builder::new()
        .suffix(&format!(".{}", suffix))
        .tempfile()?;

    file.write_all(&image_data)?;

    file.seek(std::io::SeekFrom::Start(0))?;

    debug!("File path: {}", file.path().to_str().unwrap_or_default());

    match catbox::file::from_file(file.path().to_str().unwrap_or_default(), Some(&token)).await {
        Ok(_) => {}
        Err(e) => return Err(anyhow!("failed to upload to catbox {}", e)),
    };

    Ok(())
}
