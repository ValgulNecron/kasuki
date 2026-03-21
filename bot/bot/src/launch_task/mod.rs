pub mod bot_info_update;
pub mod db_cleanup;
pub mod game_management;
pub mod ping_manager;
pub mod queue_publisher;
pub mod user_blacklist;

use crate::event_handler::BotData;
use serenity::all::Context as SerenityContext;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

use self::bot_info_update::update_bot_info;
use self::db_cleanup::db_cleanup_task;
use self::game_management::launch_game_management_thread;
use self::ping_manager::ping_manager_thread;
use self::user_blacklist::update_user_blacklist;

/// Main function responsible for launching and managing all background tasks.
#[tracing::instrument(skip(ctx, bot_data), level = "info")]
pub async fn thread_management_launcher(ctx: SerenityContext, bot_data: Arc<BotData>) {
	debug!("Preparing shared resources for background tasks");

	let apps = bot_data.apps.clone();
	let steam_cache = bot_data.steam_cache.clone();
	let user_blacklist_server_image = bot_data.user_blacklist.clone();
	let db_connection = bot_data.db_connection.clone();
	let task_intervals = bot_data.config.task_intervals.clone();
	let shutdown_signal = bot_data.shutdown_signal.clone();

	debug!("Setting up shutdown signal receivers for background tasks");
	// Collect JoinHandles so we can await all tasks on shutdown if needed
	let mut shutdown_receivers = Vec::new();

	debug!(
		"Task intervals configuration: game_update={}s, ping_update={}s, bot_info_update={}s, blacklisted_user_update={}s, server_image_update={}s, before_server_image={}s",
		task_intervals.game_update,
		task_intervals.ping_update,
		task_intervals.bot_info_update,
		task_intervals.blacklisted_user_update,
		task_intervals.server_image_update,
		task_intervals.before_server_image
	);

	info!("Launching user interaction background tasks");

	debug!(
		"Spawning game management task (interval: {}s)",
		task_intervals.game_update
	);
	let task_intervals_c = task_intervals.clone();
	let mut game_shutdown_rx = shutdown_signal.subscribe();
	let game_task = tokio::spawn(async move {
		// select! races the task against the shutdown signal — whichever completes
		// first wins, and the other branch is cancelled. This ensures each infinite-loop
		// task can be stopped without requiring cooperative cancellation points inside it.
		tokio::select! {
			_ = launch_game_management_thread(apps, task_intervals_c, steam_cache) => {
				info!("Game management task completed");
			},
			_ = game_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating game management task gracefully");
			}
		}
	});
	shutdown_receivers.push(game_task);

	info!("Launching bot status monitoring background tasks");

	debug!(
		"Spawning ping manager task (interval: {}s)",
		task_intervals.ping_update
	);
	let task_intervals_c = task_intervals.clone();
	let ctx_c = ctx.clone();
	let db_connection_c = db_connection.clone();
	let mut ping_shutdown_rx = shutdown_signal.subscribe();
	let ping_task = tokio::spawn(async move {
		tokio::select! {
			result = ping_manager_thread(ctx_c, db_connection_c, task_intervals_c) => {
				match result {
					Ok(_) => info!("Ping manager task completed successfully"),
					Err(e) => tracing::error!("Ping manager task failed: {:#}", e),
				}
			},
			_ = ping_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating ping manager task gracefully");
			}
		}
	});
	shutdown_receivers.push(ping_task);

	debug!(
		"Spawning bot info update task (interval: {}s)",
		task_intervals.bot_info_update
	);
	let task_intervals_c = task_intervals.clone();
	let mut bot_info_shutdown_rx = shutdown_signal.subscribe();
	let ctx_c = ctx.clone();
	let bot_info_c = bot_data.bot_info.clone();
	let bot_info_task = tokio::spawn(async move {
		tokio::select! {
			_ = update_bot_info(ctx_c, bot_info_c, task_intervals_c) => {
				info!("Bot info update task completed");
			},
			_ = bot_info_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating bot info update task gracefully");
			}
		}
	});
	shutdown_receivers.push(bot_info_task);

	info!("Launching security background tasks");

	debug!(
		"Spawning user blacklist update task (interval: {}s)",
		task_intervals.blacklisted_user_update
	);
	let task_intervals_c = task_intervals.clone();
	let mut blacklist_shutdown_rx = shutdown_signal.subscribe();
	let blacklist_task = tokio::spawn(async move {
		tokio::select! {
			_ = update_user_blacklist(user_blacklist_server_image.clone(), task_intervals_c) => {
				info!("User blacklist update task completed");
			},
			_ = blacklist_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating user blacklist update task gracefully");
			}
		}
	});
	shutdown_receivers.push(blacklist_task);

	info!("Launching database maintenance background tasks");

	debug!(
		"Spawning DB cleanup task (interval: {}h, retention: {}d)",
		task_intervals.db_cleanup_interval_hours, task_intervals.db_retention_days
	);
	let task_intervals_c = task_intervals.clone();
	let db_connection_c = db_connection.clone();
	let mut db_cleanup_shutdown_rx = shutdown_signal.subscribe();
	let db_cleanup = tokio::spawn(async move {
		tokio::select! {
			_ = db_cleanup_task(db_connection_c, task_intervals_c) => {
				info!("DB cleanup task completed");
			},
			_ = db_cleanup_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating DB cleanup task gracefully");
			}
		}
	});
	shutdown_receivers.push(db_cleanup);

	info!(
		"Scheduling visual tasks with delay of {}s",
		task_intervals.before_server_image
	);

	debug!(
		"Waiting {}s before launching server image management task",
		task_intervals.before_server_image
	);
	// Delay image tasks so other services (DB, API) are ready before heavy image work begins
	sleep(Duration::from_secs(task_intervals.before_server_image)).await;

	let _image_config = bot_data.config.image.clone();

	debug!(
		"Spawning server image management task (interval: {}s)",
		task_intervals.server_image_update
	);

	info!("All background tasks have been successfully launched");
	debug!(
		"Registered {} background tasks with shutdown handlers",
		shutdown_receivers.len()
	);

	info!("Background task manager initialization complete");
}
