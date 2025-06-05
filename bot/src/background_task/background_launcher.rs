use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender;
use moka::future::Cache;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::Value;
use serenity::all::{Context as SerenityContext, CurrentApplicationInfo, ShardId};
use serenity::gateway::{ShardRunnerInfo, ShardRunnerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info};

use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::get_anisong_db::get_anisong;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::background_task::update_random_stats::update_random_stats_launcher;
use crate::config::{DbConfig, ImageConfig, TaskIntervalConfig};
use crate::constant::{
	TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_ACTIVITY_CHECK, TIME_BETWEEN_ANISONG_UPDATE,
	TIME_BETWEEN_BLACKLISTED_USER_UPDATE, TIME_BETWEEN_BOT_INFO, TIME_BETWEEN_GAME_UPDATE,
	TIME_BETWEEN_PING_UPDATE, TIME_BETWEEN_SERVER_IMAGE_UPDATE,
};
use crate::database::ping_history::ActiveModel;
use crate::database::prelude::PingHistory;
use crate::event_handler::BotData;
use crate::get_url;
use crate::structure::steam_game_id_struct::get_game;
/// Main function responsible for launching and managing all background tasks.
///
/// This function orchestrates the initialization and execution of various background tasks
/// that are essential for the bot's operation. Each task runs in its own tokio task to ensure
/// concurrent execution without blocking the main thread.
///
/// # Architecture Overview
/// 
/// The background tasks are organized in the following categories:
/// 
/// 1. **Data Management Tasks**:
///    - `update_anisong_db`: Updates the anisong database periodically
///    - `update_random_stats_launcher`: Updates random statistics for anime/manga
/// 
/// 2. **Bot Status Tasks**:
///    - `ping_manager_thread`: Monitors and records shard latency
///    - `update_bot_info`: Keeps bot information up-to-date
/// 
/// 3. **User Interaction Tasks**:
///    - `launch_activity_management_thread`: Manages anime activity tracking
///    - `launch_game_management_thread`: Tracks and updates game information
/// 
/// 4. **Security Tasks**:
///    - `update_user_blacklist`: Maintains a list of blacklisted users
/// 
/// 5. **Visual Tasks** (launched after a delay):
///    - `launch_server_image_management_thread`: Generates and updates server images
///
/// # Task Scheduling
/// 
/// Each task has its own configurable interval defined in `TaskIntervalConfig`.
/// This allows for fine-tuning the frequency of each background operation based on
/// its resource requirements and importance.
///
/// # Arguments
///
/// * `ctx` - Serenity context for Discord API interactions
/// * `bot_data` - Shared bot data including caches and configuration
/// * `db_config` - Database configuration
///
pub async fn thread_management_launcher(
	ctx: SerenityContext, bot_data: Arc<BotData>, db_config: DbConfig,
) {
	// Extract shared resources that will be used by multiple background tasks
	let anilist_cache = bot_data.anilist_cache.clone();
	let apps = bot_data.apps.clone();
	let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();
	let connection = bot_data.db_connection.clone();
	let task_intervals = bot_data.config.task_intervals.clone();

	// === DATA MANAGEMENT TASKS ===

	// Launch anisong database update task
	// This task periodically fetches and updates the database of anime songs
	tokio::spawn(update_anisong_db(connection.clone(), task_intervals.clone()));

	// Launch random stats update task
	// This task updates statistics about anime and manga for random commands
	tokio::spawn(update_random_stats_launcher(anilist_cache.clone(), task_intervals.clone()));

	// === USER INTERACTION TASKS ===

	// Launch activity management thread
	// This task tracks and updates anime activity for users
	tokio::spawn(launch_activity_management_thread(
		ctx.clone(),
		anilist_cache.clone(),
		db_config.clone(),
		task_intervals.clone(),
	));

	// Launch game management thread
	// This task tracks and updates game information
	tokio::spawn(launch_game_management_thread(apps, task_intervals.clone()));

	// === BOT STATUS TASKS ===

	// Launch ping manager thread
	// This task monitors and records shard latency
	tokio::spawn(ping_manager_thread(ctx.clone(), db_config.clone(), task_intervals.clone()));

	// Launch bot info update task
	// This task keeps bot information up-to-date
	tokio::spawn(update_bot_info(ctx.clone(), bot_data.bot_info.clone(), task_intervals.clone()));

	// === SECURITY TASKS ===

	// Launch user blacklist update task
	// This task maintains a list of blacklisted users
	tokio::spawn(update_user_blacklist(user_blacklist_server_image.clone(), task_intervals.clone()));

	// === VISUAL TASKS (with delay) ===

	// Wait before launching server image management
	// This delay ensures other critical tasks are running before starting image generation
	sleep(Duration::from_secs(task_intervals.before_server_image)).await;

	let image_config = bot_data.config.image.clone();

	// Launch server image management thread
	// This task generates and updates server images
	tokio::spawn(launch_server_image_management_thread(
		ctx.clone(),
		image_config,
		connection,
		task_intervals.clone(),
	));

	info!("Done spawning thread manager.");
}

async fn update_anisong_db(db: Arc<DatabaseConnection>, task_intervals: TaskIntervalConfig) {
	info!("Launching the anisongdb thread!");
	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.anisong_update));
	loop {
		interval.tick().await;
		get_anisong(db.clone()).await;
	}
}

/// Monitors and records the latency (ping) of each Discord gateway shard.
///
/// This function is responsible for periodically checking the latency of each shard
/// and recording this information in the database. This data is valuable for:
/// 1. Monitoring the health of the bot's connection to Discord
/// 2. Identifying potential network issues
/// 3. Providing historical performance data for analysis
///
/// # Shard Management Architecture
///
/// Discord bots connect to the gateway using one or more shards, where each shard
/// is responsible for a subset of guilds. This function:
/// - Retrieves the shard manager from the bot's context
/// - Periodically iterates through all active shards
/// - Records the latency of each shard in the database
///
/// # Error Handling and Recovery
///
/// The function implements several error recovery mechanisms:
/// - If the shard manager is not available, it will sleep and retry
/// - If database connection fails, it logs the error and exits
/// - If recording a specific shard's latency fails, it logs the error but continues with other shards
///
/// # Arguments
///
/// * `ctx` - Serenity context for accessing the shard manager
/// * `db_config` - Database configuration for connecting to the database
/// * `task_intervals` - Configuration for how frequently to check and record latency
///
async fn ping_manager_thread(ctx: SerenityContext, db_config: DbConfig, task_intervals: TaskIntervalConfig) {
	// Log the initialization of the ping monitoring thread
	info!("Launching the ping thread!");

	// Retrieve the shard manager from the bot's context
	// The shard manager contains information about all active shards
	let shard_manager: Arc<
		DashMap<ShardId, (ShardRunnerInfo, UnboundedSender<ShardRunnerMessage>)>,
	> = match ctx
		.data::<BotData>()
		.shard_manager
		.clone()
		.read()
		.await
		.clone()
	{
		Some(shard_manager) => shard_manager,
		None => {
			// If the shard manager is not available (which might happen during startup),
			// sleep for the configured interval and then retry by recursively calling this function
			tokio::time::sleep(Duration::from_secs(task_intervals.ping_update)).await;
			Box::pin(ping_manager_thread(ctx, db_config, task_intervals)).await;
			return;
		},
	};

	// Set up a periodic interval for checking shard latency
	// This determines how frequently we'll record ping data
	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.ping_update));

	// Establish a connection to the database for recording ping history
	// This connection is reused for all database operations to avoid repeatedly connecting
	let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
		Ok(connection) => connection,
		Err(e) => {
			error!("Failed to connect to the database. {:#?}", e);
			return;
		},
	};

	// Main monitoring loop - runs indefinitely
	loop {
		// Wait for the next scheduled check based on the configured interval
		interval.tick().await;

		// Get a reference to the shard manager
		// DashMap provides thread-safe concurrent access without explicit locking
		let runner = &shard_manager;

		// Iterate through each shard to record its latency
		for entry in runner.iter() {
			// Get the shard ID (a numeric identifier for the shard)
			let shard_id = entry.key();

			// Get the shard information
			let shard = entry.value();

			// Extract the current latency and timestamp
			// This is done in a separate scope to limit the lifetime of any temporary borrows
			let (now, latency) = {
				// Destructure to get the shard info (we don't need the sender)
				let (shard_info, _) = shard;

				// Get the latency in milliseconds, defaulting to 0 if not available
				// Convert to string for storage in the database
				let latency = shard_info
					.latency
					.unwrap_or_default()
					.as_millis()
					.to_string();

				// Get the current UTC timestamp for recording when this measurement was taken
				let now = chrono::Utc::now().naive_utc();

				(now, latency)
			};

			// Create a new database record with the shard ID, latency, and timestamp
			// Then execute the insert operation
			match PingHistory::insert(ActiveModel {
				shard_id: Set(shard_id.to_string()),
				latency: Set(latency),
				timestamp: Set(now),
				..Default::default() // Use defaults for any other fields
			})
			.exec(&connection)
			.await
			{
				Ok(_) => {
					// Log successful updates at debug level to avoid cluttering logs
					debug!("Updated ping history for shard {}.", shard_id);
				},
				Err(e) => {
					// Log failures at error level for investigation
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

async fn launch_game_management_thread(apps: Arc<RwLock<HashMap<String, u128>>>, task_intervals: TaskIntervalConfig) {
	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(task_intervals.game_update));

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
	task_intervals: TaskIntervalConfig,
) {
	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(task_intervals.activity_check));

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
	task_intervals: TaskIntervalConfig,
) {
	// Log a message indicating that the server image management thread is being launched
	info!("Launching the server image management thread!");

	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(task_intervals.server_image_update));

	// Loop indefinitely
	loop {
		// Wait for the next interval tick
		interval.tick().await;

		// Call the server_image_management function with the provided context, database type, and image configuration
		server_image_management(&ctx, image_config.clone(), connection.clone()).await;
	}
}

/// Maintains a list of blacklisted users by periodically fetching from a remote source.
///
/// This function is responsible for keeping an up-to-date list of users who are blacklisted
/// from using certain bot features. The blacklist is fetched from a GitHub repository
/// and stored in a shared data structure that can be accessed by other parts of the bot.
///
/// # Security Considerations
///
/// This blacklist mechanism provides a way to quickly block abusive users without
/// requiring a bot restart or code deployment. It's an important security feature that:
/// 
/// 1. Allows for rapid response to abuse
/// 2. Centralizes the blacklist management
/// 3. Enables updates without service interruption
///
/// # JSON Format
///
/// The expected JSON format from the remote source is:
/// ```json
/// {
///   "user_id": ["123456789", "987654321", ...]
/// }
/// ```
///
/// # Error Handling
///
/// The function includes error handling for:
/// - Network failures when fetching the blacklist
/// - JSON parsing errors
/// - Missing or malformed data in the JSON
///
/// # Arguments
///
/// * `blacklist_lock` - Thread-safe shared storage for the blacklist
/// * `task_intervals` - Configuration for how frequently to update the blacklist
///
async fn update_user_blacklist(blacklist_lock: Arc<RwLock<Vec<String>>>, task_intervals: TaskIntervalConfig) {
	// Set up a periodic interval for blacklist updates
	// This determines how frequently we'll check for changes to the blacklist
	let mut interval =
		tokio::time::interval(Duration::from_secs(task_intervals.blacklisted_user_update));

	// Main update loop - runs indefinitely
	loop {
		// Wait for the next scheduled update based on the configured interval
		interval.tick().await;

		// Define the URL where the blacklist is stored
		// Using GitHub raw content ensures we always get the latest version from the specified branch
		let blacklist_url =
			"https://raw.githubusercontent.com/ValgulNecron/kasuki/dev/blacklist.json";

		// Fetch the blacklist data from the remote URL
		// Note: This uses expect() which will panic on failure - in a production environment,
		// this should be replaced with proper error handling to prevent the thread from crashing
		let blacklist_response = reqwest::get(blacklist_url)
			.await
			.expect("Failed to get blacklist");

		// Parse the JSON response into a serde_json Value
		// This generic Value type allows us to navigate the JSON structure
		let blacklist_json: Value = blacklist_response
			.json()
			.await
			.expect("Failed to parse blacklist");

		// Extract user IDs from the JSON array at the "user_id" key
		// This handles the case where the JSON structure might not match our expectations
		let user_ids: Vec<String> = match blacklist_json["user_id"].as_array() {
			Some(arr) => arr, // If we found an array at the "user_id" key, use it
			None => {
				// If there's no array at "user_id", log an error and skip this update
				error!("Failed to get user_id from blacklist");
				continue;
			},
		}
		// Convert each element in the array to a String
		.iter()
		.map(|id| match id.as_str() {
			Some(id) => id.to_string(), // If the element is a string, convert it
			None => {
				// If the element is not a string, log an error and use an empty string
				// This ensures we don't crash even if some elements are invalid
				error!("Failed to get user_id from blacklist");
				"".to_string()
			},
		})
		.collect();

		// Update the shared blacklist with the new data
		// This requires acquiring a write lock on the shared data
		let mut blacklist = blacklist_lock.write().await;

		// Clear the existing blacklist to remove any users who are no longer blacklisted
		blacklist.clear();

		// Shrink the capacity to match the new size
		// This frees memory if the blacklist has significantly decreased in size
		blacklist.shrink_to_fit();

		// Replace the blacklist with the new list of user IDs
		*blacklist = user_ids;

		// The lock is automatically released when `blacklist` goes out of scope
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
	task_intervals: TaskIntervalConfig,
) {
	// Create a time interval for updating bot info
	let mut update_interval = tokio::time::interval(Duration::from_secs(task_intervals.bot_info));

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
