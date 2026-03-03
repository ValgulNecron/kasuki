use anyhow::{Context, Result};
use redis::AsyncCommands;
use tracing::debug;

use super::tasks::ImageTask;

pub const SERVER_IMAGE_QUEUE_KEY: &str = "image_generation:server_image";
pub const USER_COLOR_QUEUE_KEY: &str = "image_generation:user_color";

pub async fn publish_task(
	connection: &mut redis::aio::MultiplexedConnection, key: &str, task: &ImageTask,
) -> Result<()> {
	let payload = serde_json::to_string(task).context("Failed to serialize ImageTask")?;
	debug!("Publishing task to {}: {} bytes", key, payload.len());
	connection
		.rpush::<_, _, ()>(key, &payload)
		.await
		.context("Failed to rpush task to Redis")?;
	Ok(())
}
