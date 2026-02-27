use anyhow::{Context, Result};
use redis::AsyncCommands;
use tracing::debug;

use super::tasks::ImageTask;

pub const QUEUE_KEY: &str = "image_generation:tasks";

pub async fn publish_task(
	connection: &mut redis::aio::MultiplexedConnection, task: &ImageTask,
) -> Result<()> {
	let payload = serde_json::to_string(task).context("Failed to serialize ImageTask")?;
	debug!("Publishing task to {}: {} bytes", QUEUE_KEY, payload.len());
	connection
		.rpush::<_, _, ()>(QUEUE_KEY, &payload)
		.await
		.context("Failed to rpush task to Redis")?;
	Ok(())
}
