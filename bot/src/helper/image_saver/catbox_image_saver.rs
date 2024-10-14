use std::error::Error;
use std::io::{Seek, Write};
use tracing::debug;

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

    Ok(())
}
