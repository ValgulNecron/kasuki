use std::sync::Arc;

use shared::queue::publisher::{publish_task, SERVER_IMAGE_QUEUE_KEY, USER_COLOR_QUEUE_KEY};
use shared::queue::tasks::ImageTask;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info};

use crate::event_handler::BotData;

pub async fn user_color_queue_publisher(
	mut rx: UnboundedReceiver<ImageTask>, bot_data: Arc<BotData>,
) {
	info!("User color queue publisher started");

	while let Some(task) = rx.recv().await {
		let mut guard = match bot_data.get_redis_connection().await {
			Some(g) => g,
			None => {
				error!("Redis unavailable, dropping user color task");
				continue;
			},
		};
		if let Err(e) = publish_task(guard.as_mut().unwrap(), USER_COLOR_QUEUE_KEY, &task).await {
			error!("Failed to publish user color task: {:#}", e);
		}
		drop(task);
	}

	info!("User color queue publisher stopped");
}

pub async fn server_image_queue_publisher(
	mut rx: UnboundedReceiver<ImageTask>, bot_data: Arc<BotData>,
) {
	info!("Server image queue publisher started");

	while let Some(task) = rx.recv().await {
		let mut guard = match bot_data.get_redis_connection().await {
			Some(g) => g,
			None => {
				error!("Redis unavailable, dropping server image task");
				continue;
			},
		};
		if let Err(e) = publish_task(guard.as_mut().unwrap(), SERVER_IMAGE_QUEUE_KEY, &task).await {
			error!("Failed to publish server image task: {:#}", e);
		}
		drop(task);
	}

	info!("Server image queue publisher stopped");
}
