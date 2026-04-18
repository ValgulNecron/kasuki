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
use tracing::info;

use self::bot_info_update::update_bot_info;
use self::db_cleanup::db_cleanup_task;
use self::game_management::launch_game_management_thread;
use self::ping_manager::ping_manager_thread;
use self::user_blacklist::update_user_blacklist;

/// Main function responsible for launching and managing all background tasks.
#[tracing::instrument(skip(ctx, bot_data), level = "info")]
pub async fn thread_management_launcher(ctx: SerenityContext, bot_data: Arc<BotData>) {
	let apps = bot_data.apps.clone();
	let steam_cache = bot_data.steam_cache.clone();
	let user_blacklist_server_image = bot_data.user_blacklist.clone();
	let db_connection = bot_data.db_connection.clone();
	let task_intervals = bot_data.config.task_intervals.clone();
	let shutdown_signal = bot_data.shutdown_signal.clone();

	let mut shutdown_receivers = Vec::new();

	let task_intervals_c = task_intervals.clone();
	let mut game_shutdown_rx = shutdown_signal.subscribe();
	let game_task = tokio::spawn(async move {
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

	sleep(Duration::from_secs(task_intervals.before_server_image)).await;

	let _image_config = bot_data.config.image.clone();

	info!("All background tasks launched");
}
