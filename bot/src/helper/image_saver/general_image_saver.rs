use anyhow::{Context, Result};
use tracing::{debug, error, info, trace, warn};

// Importing necessary libraries and modules
use crate::helper::image_saver::catbox_image_saver::upload_image_catbox;
use crate::helper::image_saver::local_image_saver::local_image_save;

/// Saves an image either locally or remotely based on the specified save type.
///
/// This function acts as a facade for different image saving strategies, abstracting
/// the details of where and how the image is stored. It supports two primary storage
/// strategies:
///
/// 1. Local storage: Saves the image to the local filesystem, organized by guild ID
/// 2. Remote storage: Uploads the image to a remote service (currently only Catbox)
///
/// # Parameters
///
/// * `guild_id` - The Discord guild ID associated with the image
/// * `filename` - The name to give the saved image file
/// * `image_data` - The raw binary image data as a byte vector
/// * `saver_server` - The remote server to use (only relevant for remote saves)
/// * `token` - Authentication token for the remote service (only relevant for remote saves)
/// * `save_type` - Storage strategy to use ("local" or "remote")
///
/// # Error Handling
///
/// The function provides detailed error context for debugging purposes, including
/// the guild ID and filename for local saves, and the server and filename for remote saves.
///
/// # Storage Strategy Selection
///
/// The storage strategy is determined by the `save_type` parameter:
/// - "local": Saves to the local filesystem
/// - "remote": Uploads to a remote service
/// - Any other value: No action is taken (returns success)
///
pub async fn image_saver(
	guild_id: String, filename: String, image_data: Vec<u8>, saver_server: String, token: String,
	save_type: String,
) -> Result<()> {
	info!("Saving image: {}", filename);
	debug!("Image save parameters: guild_id={}, save_type={}, server={}", guild_id, save_type, saver_server);
	debug!("Image data size: {} bytes", image_data.len());

	// Strategy pattern: Select the appropriate saving strategy based on save_type
	if save_type == *"local" {
		trace!("Using local image saving method");
		info!("Saving image locally for guild: {}", guild_id);

		match local_image_save(guild_id.clone(), filename.clone(), image_data).await {
			Ok(_) => {
				info!("Successfully saved image locally: {}", filename);
				Ok(())
			},
			Err(e) => {
				error!("Failed to save image locally: {}", e);
				Err(e).with_context(|| format!("Failed to save image locally for guild: {}, filename: {}", guild_id, filename))
			}
		}
	} else if save_type == *"remote" {
		trace!("Using remote image saving method");
		info!("Saving image remotely to server: {}", saver_server);

		match remote_saver(filename.clone(), image_data, saver_server.clone(), token).await {
			Ok(_) => {
				info!("Successfully saved image remotely: {}", filename);
				Ok(())
			},
			Err(e) => {
				error!("Failed to save image remotely: {}", e);
				Err(e).with_context(|| format!("Failed to save image remotely, filename: {}, server: {}", filename, saver_server))
			}
		}
	} else {
		// Graceful handling of unknown save types
		// This allows for future expansion of save types without breaking existing code
		warn!("Unknown save_type: {}, no action taken", save_type);
		Ok(())
	}
}

/// Uploads an image to a remote service.
///
/// This function handles the details of uploading an image to a specific remote service.
/// Currently, it only supports Catbox as a remote service, but the design allows for
/// easy extension to support additional services in the future.
///
/// # Parameters
///
/// * `filename` - The name to give the uploaded image file
/// * `image_data` - The raw binary image data as a byte vector
/// * `saver_server` - The remote server to use (currently only Catbox is supported)
/// * `token` - Authentication token for the remote service
///
/// # Service Selection
///
/// The function uses a match statement on `saver_server` to select the appropriate
/// upload implementation. This design makes it easy to add support for additional
/// remote services in the future by adding new match arms.
///
/// # Future Extensibility
///
/// To add support for a new remote service:
/// 1. Create a new module with the upload implementation
/// 2. Add a new match arm in this function for the new service
/// 3. Call the appropriate upload function from the new module
///
pub async fn remote_saver(
	filename: String, image_data: Vec<u8>, saver_server: String, token: String,
) -> Result<()> {
	trace!("Starting remote image save process");
	debug!("Remote server: {}", saver_server);
	debug!("Image filename: {}", filename);
	debug!("Image data size: {} bytes", image_data.len());

	// Factory pattern: Select the appropriate upload implementation based on saver_server
	match saver_server.as_str() {
		// Currently, we only support Catbox, but this design allows for easy extension
		// to support additional services in the future
		_ => {
			info!("Using Catbox as image upload service");
			debug!("Uploading image to Catbox: {}", filename);

			match upload_image_catbox(filename.clone(), image_data, token).await {
				Ok(_) => {
					info!("Successfully uploaded image to Catbox: {}", filename);
					Ok(())
				},
				Err(e) => {
					error!("Failed to upload image to Catbox: {}", e);
					Err(e).with_context(|| format!("Failed to upload image to Catbox, filename: {}", filename))
				}
			}
		}
	}
}
