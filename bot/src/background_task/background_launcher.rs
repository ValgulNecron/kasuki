use std::collections::HashMap;
use std::sync::{Arc, PoisonError};
use std::time::Duration;
use anyhow::anyhow;
use moka::future::Cache;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::Value;
use serenity::all::{Context as SerenityContext, CurrentApplicationInfo, ShardRunnerInfo};
use tokio::sync::{MutexGuard, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info};

use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::get_anisong_db::get_anisong;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::background_task::update_random_stats::update_random_stats_launcher;
use crate::config::{DbConfig, ImageConfig};
use crate::constant::{
	TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_ACTIVITY_CHECK, TIME_BETWEEN_BLACKLISTED_USER_UPDATE,
	TIME_BETWEEN_BOT_INFO, TIME_BETWEEN_GAME_UPDATE, TIME_BETWEEN_PING_UPDATE,
	TIME_BETWEEN_SERVER_IMAGE_UPDATE,
};
use crate::database::ping_history::ActiveModel;
use crate::database::prelude::PingHistory;
use crate::event_handler::BotData;
use crate::get_url;
use crate::structure::steam_game_id_struct::get_game;
pub async fn thread_management_launcher(
	ctx: SerenityContext, bot_data: Arc<BotData>, db_config: DbConfig,
) {
	let anilist_cache = bot_data.anilist_cache.clone();

	let apps = bot_data.apps.clone();

	let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();

	let connection = bot_data.db_connection.clone();

	//tokio::spawn(update_anisong_db(connection.clone()));

	tokio::spawn(launch_activity_management_thread(
		ctx.clone(),
		anilist_cache.clone(),
		db_config.clone(),
	));

	tokio::spawn(launch_game_management_thread(apps));

	tokio::spawn(ping_manager_thread(ctx.clone(), db_config.clone()));

	tokio::spawn(update_user_blacklist(user_blacklist_server_image.clone()));

	tokio::spawn(update_random_stats_launcher(anilist_cache.clone()));

	tokio::spawn(update_bot_info(ctx.clone(), bot_data.bot_info.clone()));

	sleep(Duration::from_secs(1)).await;

	sleep(Duration::from_secs(TIME_BEFORE_SERVER_IMAGE)).await;

	let image_config = bot_data.config.image.clone();

	tokio::spawn(launch_server_image_management_thread(
		ctx.clone(),
		image_config,
		connection,
	));

	info!("Done spawning thread manager.");
}

async fn update_anisong_db(db: Arc<DatabaseConnection>) {
	info!("Launching the anisongdb thread!");
	let mut interval = tokio::time::interval(Duration::from_secs(TIME_BETWEEN_PING_UPDATE));
	loop {
		interval.tick().await;
		get_anisong(db.clone()).await;
	}
}

async fn ping_manager_thread(ctx: SerenityContext, db_config: DbConfig) {
	// Log a message indicating that the ping thread is being launched
	info!("Launching the ping thread!");

	// Get the ShardManager from the data
	let shard_manager = match ctx
		.data::<BotData>()
		.shard_manager
		.clone()
		.read()
		.await
		.clone()
	{
		Some(shard_manager) => shard_manager,
		None => {
			tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PING_UPDATE)).await;
			Box::pin(ping_manager_thread(ctx, db_config)).await;
			return;
		},
	};

	// Define an interval for periodic updates
	let mut interval = tokio::time::interval(Duration::from_secs(TIME_BETWEEN_PING_UPDATE));

	// Main loop for managing pings
	let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
		Ok(connection) => connection,
		Err(e) => {
			error!("Failed to connect to the database. {:#?}", e);

			return;
		},
	};

	loop {
		// Wait for the next interval tick
		interval.tick().await;

		// Lock the shard manager and iterate over the runners
		let runner = &shard_manager;

		for (shard_id, (shard)) in runner.iter() {
			// Extract latency and current timestamp information
			let (now, latency) = {
				let shard_info = match shard.lock() {
					Ok(shard_runner_info) => {shard_runner_info}
					Err(_) => {
						error!("failed to get the shard runner info");
						continue
					}
				};

				let latency = shard_info
					.latency
					.unwrap_or_default()
					.as_millis()
					.to_string();
				drop(shard_info);
				let now = chrono::Utc::now().naive_utc();
				(now, latency)
			};

			match PingHistory::insert(ActiveModel {
				shard_id: Set(shard_id.to_string()),
				latency: Set(latency),
				timestamp: Set(now),
				..Default::default()
			})
			.exec(&connection)
			.await
			{
				Ok(_) => {
					debug!("Updated ping history for shard {}.", shard_id);
				},
				Err(e) => {
					error!(
						"Failed to update ping history for shard {}. {:#?}",
						shard_id, e
					);
				},
			}
		}
	}
}

/// Asynchronously launches the game management thread.
///
/// # Arguments
///
/// * `apps` - An `Arc` wrapped `RwLock` containing a `HashMap` of `String` keys and `u128` values.
///

async fn launch_game_management_thread(apps: Arc<RwLock<HashMap<String, u128>>>) {
	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(TIME_BETWEEN_GAME_UPDATE));

	// Log a message indicating that the steam management thread is being launched
	info!("Launching the steam management thread!");

	// Loop indefinitely
	loop {
		// Wait for the next interval tick
		interval.tick().await;

		// Get the game with the provided apps and wait for the result
		get_game(apps.clone()).await;
	}
}

/// This function is responsible for launching the activity management thread.
/// It takes a `Context` and a `String` as arguments, and does not return anything.
///
/// The `Context` is used to access the bot's data and cache.
/// The `String` is the type of database being used.
///
/// The function creates an interval for periodic updates and logs a message indicating that the thread is being launched.
/// It then enters a loop that waits for the next interval tick and spawns a new task to manage the bot's activity.
/// The task is cloned from the `Context` and `db_type` arguments.
///

async fn launch_activity_management_thread(
	ctx: SerenityContext, anilist_cache: Arc<RwLock<Cache<String, String>>>, db_config: DbConfig,
) {
	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(TIME_BETWEEN_ACTIVITY_CHECK));

	// Log a message indicating that the activity management thread is being launched
	info!("Launching the activity management thread!");

	// Enter a loop that waits for the next interval tick and spawns a new task to manage the bot's activity
	loop {
		// Wait for the next interval tick
		interval.tick().await;

		// Clone the context and db_type arguments
		let ctx = ctx.clone();

		// Spawn a new task to manage the bot's activity
		tokio::spawn(manage_activity(
			ctx,
			anilist_cache.clone(),
			db_config.clone(),
		));
	}
}

/// This function is responsible for launching the server image management thread.
/// It takes a `Context` as an argument.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used in the server image management function.
///

async fn launch_server_image_management_thread(
	ctx: SerenityContext, image_config: ImageConfig, connection: Arc<DatabaseConnection>,
) {
	// Log a message indicating that the server image management thread is being launched
	info!("Launching the server image management thread!");

	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(TIME_BETWEEN_SERVER_IMAGE_UPDATE));

	// Loop indefinitely
	loop {
		// Wait for the next interval tick
		interval.tick().await;

		// Call the server_image_management function with the provided context, database type, and image configuration
		server_image_management(&ctx, image_config.clone(), connection.clone()).await;
	}
}

/// Asynchronously updates the user blacklist based on the retrieved data from a URL.
///
/// # Arguments
///
/// * `blacklist_lock` - An `Arc` wrapped `RwLock` containing the blacklist data.
///

async fn update_user_blacklist(blacklist_lock: Arc<RwLock<Vec<String>>>) {
	// Create an interval for periodic updates
	let mut interval =
		tokio::time::interval(Duration::from_secs(TIME_BETWEEN_BLACKLISTED_USER_UPDATE));

	loop {
		// Wait for the interval to tick
		interval.tick().await;

		// Fetch the blacklist data from a URL
		let blacklist_url =
			"https://raw.githubusercontent.com/ValgulNecron/kasuki/dev/blacklist.json";

		let blacklist_response = reqwest::get(blacklist_url)
			.await
			.expect("Failed to get blacklist");

		// Parse the JSON response into a Value type
		let blacklist_json: Value = blacklist_response
			.json()
			.await
			.expect("Failed to parse blacklist");

		// Extract user IDs from the JSON array
		let user_ids: Vec<String> = match blacklist_json["user_id"].as_array() {
			Some(arr) => arr,
			None => {
				error!("Failed to get user_id from blacklist");

				continue;
			},
		}
		.iter()
		.map(|id| match id.as_str() {
			Some(id) => id.to_string(),
			None => {
				error!("Failed to get user_id from blacklist");

				"".to_string()
			},
		})
		.collect();

		// Write the updated blacklist to the shared data structure
		let mut blacklist = blacklist_lock.write().await;

		blacklist.clear();

		blacklist.shrink_to_fit();

		*blacklist = user_ids;
	}
}

/// Asynchronously updates the bot information based on the context and bot data.
///
/// # Arguments
///
/// * `context` - A `Context` instance used to retrieve information.
/// * `bot_data` - An `Arc` reference to the `BotData` struct.
///

async fn update_bot_info(
	context: SerenityContext, bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
) {
	// Create a time interval for updating bot info
	let mut update_interval = tokio::time::interval(Duration::from_secs(TIME_BETWEEN_BOT_INFO));

	loop {
		// Wait for the update interval
		update_interval.tick().await;

		// Retrieve the current bot information
		let current_bot_info = match context.http.get_current_application_info().await {
			Ok(info) => info,
			Err(e) => {
				error!("Failed to get bot info: {:?}", e);

				continue;
			},
		};

		// Acquire a lock on bot info and update it with the current information
		let mut bot_info_lock = bot_info.write().await;

		*bot_info_lock = Some(current_bot_info);
	}
}
