use serenity::all::{Context as SerenityContext, CurrentApplicationInfo};
use shared::config::TaskIntervalConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::error;

/// Asynchronously updates the bot information based on the context and bot data.
#[tracing::instrument(skip(context, bot_info, task_intervals), level = "debug")]
pub async fn update_bot_info(
	context: SerenityContext, bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
	task_intervals: TaskIntervalConfig,
) {
	let mut update_interval = tokio::time::interval(Duration::from_secs(task_intervals.bot_info));

	loop {
		update_interval.tick().await;

		let current_bot_info = match context.http.get_current_application_info().await {
			Ok(info) => info,
			Err(e) => {
				error!("Failed to get bot info: {:?}", e);

				continue;
			},
		};

		let mut bot_info_lock = bot_info.write().await;

		*bot_info_lock = Some(current_bot_info);
	}
}
