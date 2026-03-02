use anyhow::{Context, Result};
use tracing::{debug, error, info, trace, warn};

use crate::image_saver::catbox_image_saver::upload_image_catbox;
use crate::image_saver::local_image_saver::local_image_save;

pub async fn image_saver(
	guild_id: String, filename: String, image_data: Vec<u8>, saver_server: String, token: String,
	save_type: String,
) -> Result<()> {
	info!("Saving image: {}", filename);
	debug!(
		"guild_id={}, save_type={}, server={}, {} bytes",
		guild_id,
		save_type,
		saver_server,
		image_data.len()
	);

	if save_type == *"local" {
		local_image_save(guild_id.clone(), filename.clone(), image_data)
			.await
			.with_context(|| {
				format!(
					"Failed to save image locally for guild: {}, filename: {}",
					guild_id, filename
				)
			})
	} else if save_type == *"remote" {
		remote_saver(filename.clone(), image_data, saver_server.clone(), token)
			.await
			.with_context(|| {
				format!(
					"Failed to save image remotely, filename: {}, server: {}",
					filename, saver_server
				)
			})
	} else {
		warn!("Unknown save_type: {}, no action taken", save_type);
		Ok(())
	}
}

pub async fn remote_saver(
	filename: String, image_data: Vec<u8>, saver_server: String, token: String,
) -> Result<()> {
	trace!(
		"Remote save: server={}, filename={}, {} bytes",
		saver_server,
		filename,
		image_data.len()
	);

	match saver_server.as_str() {
		_ => upload_image_catbox(filename.clone(), image_data, token)
			.await
			.with_context(|| format!("Failed to upload image to Catbox, filename: {}", filename)),
	}
}
