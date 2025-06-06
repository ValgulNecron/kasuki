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
use tokio::sync::broadcast;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, trace, warn};

use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::get_anisong_db::get_anisong;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::background_task::update_random_stats::update_random_stats_launcher;
use crate::config::{DbConfig, ImageConfig, TaskIntervalConfig};
use crate::database::ping_history::ActiveModel;
use crate::database::prelude::PingHistory;
use crate::event_handler::BotData;
use crate::get_url;
use anyhow::{Context as AnyhowContext, Result};
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
	info!("Initializing background task manager");
	debug!("Preparing shared resources for background tasks");

	// Extract shared resources that will be used by multiple background tasks
	let anilist_cache = bot_data.anilist_cache.clone();
	let apps = bot_data.apps.clone();
	let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();
	let connection = bot_data.db_connection.clone();
	let task_intervals = bot_data.config.task_intervals.clone();
	let shutdown_signal = bot_data.shutdown_signal.clone();

	debug!("Setting up shutdown signal receivers for background tasks");
	let mut shutdown_receivers = Vec::new();

	// Log task interval configuration for debugging
	debug!("Task intervals configuration: anisong_update={}s, random_stats_update={}s, activity_check={}s, game_update={}s, ping_update={}s, bot_info_update={}s, blacklisted_user_update={}s, server_image_update={}s, before_server_image={}s",
		task_intervals.anisong_update,
		task_intervals.random_stats_update,
		task_intervals.activity_check,
		task_intervals.game_update,
		task_intervals.ping_update,
		task_intervals.bot_info_update,
		task_intervals.blacklisted_user_update,
		task_intervals.server_image_update,
		task_intervals.before_server_image
	);

	// === DATA MANAGEMENT TASKS ===
	info!("Launching data management background tasks");

	// Launch anisong database update task
	debug!("Spawning anisong database update task (interval: {}s)", task_intervals.anisong_update);
	let task_intervals_c = task_intervals.clone();
	let connection_c = connection.clone();
	let mut anisong_shutdown_rx = shutdown_signal.subscribe();
	let anisong_task = tokio::spawn(async move {
		tokio::select! {
			_ = update_anisong_db(connection_c, task_intervals_c) => {
				info!("Anisong database update task completed");
			},
			_ = anisong_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating anisong database update task gracefully");
			}
		}
	});
	shutdown_receivers.push(anisong_task);

	// Launch random stats update task
	debug!("Spawning random stats update task (interval: {}s)", task_intervals.random_stats_update);
	let task_intervals_c = task_intervals.clone();
	let anilist_cache_c = anilist_cache.clone();
	let mut random_stats_shutdown_rx = shutdown_signal.subscribe();
	let random_stats_task = tokio::spawn(async move {
		tokio::select! {
			_ = update_random_stats_launcher(anilist_cache_c, task_intervals_c) => {
				info!("Random stats update task completed");
			},
			_ = random_stats_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating random stats update task gracefully");
			}
		}
	});
	shutdown_receivers.push(random_stats_task);

	// === USER INTERACTION TASKS ===
	info!("Launching user interaction background tasks");

	// Launch activity management thread
	debug!("Spawning activity management task (interval: {}s)", task_intervals.activity_check);
	let task_intervals_c = task_intervals.clone();
	let ctx_c =ctx.clone();
	let anilist_cache_c = anilist_cache.clone();
	let db_config_c = db_config.clone();
	let mut activity_shutdown_rx = shutdown_signal.subscribe();
	let activity_task = tokio::spawn(async move {
		tokio::select! {
			_ = launch_activity_management_thread(
				ctx_c,
				anilist_cache_c,
				db_config_c,
				task_intervals_c,
			) => {
				info!("Activity management task completed");
			},
			_ = activity_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating activity management task gracefully");
			}
		}
	});
	shutdown_receivers.push(activity_task);

	// Launch game management thread
	debug!("Spawning game management task (interval: {}s)", task_intervals.game_update);
	let task_intervals_c = task_intervals.clone();
	let mut game_shutdown_rx = shutdown_signal.subscribe();
	let game_task = tokio::spawn(async move {
		tokio::select! {
			_ = launch_game_management_thread(apps, task_intervals_c) => {
				info!("Game management task completed");
			},
			_ = game_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating game management task gracefully");
			}
		}
	});
	shutdown_receivers.push(game_task);

	// === BOT STATUS TASKS ===
	info!("Launching bot status monitoring background tasks");

	// Launch ping manager thread
	debug!("Spawning ping manager task (interval: {}s)", task_intervals.ping_update);
	let task_intervals_c = task_intervals.clone();
	let ctx_c = ctx.clone();
	let db_config_c = db_config.clone();
	let mut ping_shutdown_rx = shutdown_signal.subscribe();
	let ping_task = tokio::spawn(async move {
		tokio::select! {
			result = ping_manager_thread(ctx_c, db_config_c, task_intervals_c) => {
				match result {
					Ok(_) => info!("Ping manager task completed successfully"),
					Err(e) => error!("Ping manager task failed: {:#}", e),
				}
			},
			_ = ping_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating ping manager task gracefully");
			}
		}
	});
	shutdown_receivers.push(ping_task);

	// Launch bot info update task
	debug!("Spawning bot info update task (interval: {}s)", task_intervals.bot_info_update);
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

	// === SECURITY TASKS ===
	info!("Launching security background tasks");

	// Launch user blacklist update task
	debug!("Spawning user blacklist update task (interval: {}s)", task_intervals.blacklisted_user_update);
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

	// === VISUAL TASKS (with delay) ===
	info!("Scheduling visual tasks with delay of {}s", task_intervals.before_server_image);

	// Wait before launching server image management
	// This delay ensures other critical tasks are running before starting image generation
	debug!("Waiting {}s before launching server image management task", task_intervals.before_server_image);
	sleep(Duration::from_secs(task_intervals.before_server_image)).await;

	let image_config = bot_data.config.image.clone();

	// Launch server image management thread
	debug!("Spawning server image management task (interval: {}s)", task_intervals.server_image_update);
	let task_intervals_c = task_intervals.clone();
	let ctx_c =ctx.clone();
	let mut server_image_shutdown_rx = shutdown_signal.subscribe();
	let server_image_task = tokio::spawn(async move {
		tokio::select! {
			_ = launch_server_image_management_thread(
				ctx_c,
				image_config,
				connection,
				task_intervals_c,
			) => {
				info!("Server image management task completed");
			},
			_ = server_image_shutdown_rx.recv() => {
				info!("Received shutdown signal, terminating server image management task gracefully");
			}
		}
	});
	shutdown_receivers.push(server_image_task);

	info!("All background tasks have been successfully launched");
	debug!("Registered {} background tasks with shutdown handlers", shutdown_receivers.len());

	// Return the JoinHandles so they're not dropped
	// This keeps the tasks running until they receive a shutdown signal
	info!("Background task manager initialization complete");
}

/// Background task that periodically updates the anime song database.
///
/// This function runs in an infinite loop, periodically fetching anime song data
/// from the anisongdb.com API and storing it in the database. The update interval
/// is configurable through the `task_intervals.anisong_update` setting.
///
/// # Arguments
///
/// * `db` - An Arc-wrapped database connection
/// * `task_intervals` - Configuration for task intervals
///
async fn update_anisong_db(db: Arc<DatabaseConnection>, task_intervals: TaskIntervalConfig) {
	info!("Launching the anisongdb update background task");
	debug!("Anisong database will update every {} seconds", task_intervals.anisong_update);

	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.anisong_update));
	let mut update_count = 0;
	let mut consecutive_failures = 0;

	loop {
		// Wait for the next scheduled update time
		trace!("Waiting for next anisong update interval tick");
		interval.tick().await;
		update_count += 1;

		info!("Starting anisong database update cycle #{}", update_count);
		debug!("Current time: {}", chrono::Utc::now());

		// Attempt to update the anisong database
		trace!("Calling get_anisong function with database connection");
		match get_anisong(db.clone()).await {
			Ok(count) => {
				info!("Anisong database update cycle #{} completed successfully", update_count);
				debug!("Updated {} anisong records in the database", count);

				// Reset failure counter on success
				if consecutive_failures > 0 {
					debug!("Reset consecutive failure counter from {} to 0", consecutive_failures);
					consecutive_failures = 0;
				}
			},
			Err(err) => {
				consecutive_failures += 1;
				error!(
					"Anisong database update cycle #{} failed: {:#}",
					update_count, err
				);
				warn!(
					"This is consecutive failure #{} for anisong updates",
					consecutive_failures
				);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&err));
				debug!("Error context: {}", err.to_string());

				// Continue with the next scheduled update despite the error
				if consecutive_failures >= 3 {
					warn!("Multiple consecutive anisong update failures detected. Consider checking the database connection or API availability.");
				}
			}
		}

		debug!("Next anisong database update scheduled in {} seconds", task_intervals.anisong_update);
		trace!("Update cycle #{} completed at {}", update_count, chrono::Utc::now());
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
async fn ping_manager_thread(ctx: SerenityContext, db_config: DbConfig, task_intervals: TaskIntervalConfig) -> Result<()> {
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
			Box::pin(ping_manager_thread(ctx, db_config, task_intervals)).await?;
			return Ok(());
		},
	};

	// Set up a periodic interval for checking shard latency
	// This determines how frequently we'll record ping data
	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.ping_update));

	// Establish a connection to the database for recording ping history
	// This connection is reused for all database operations to avoid repeatedly connecting
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await
		.context(format!("Failed to connect to database with config: {:?}", db_config))?;

	// Main monitoring loop - runs indefinitely
	loop {
		// Wait for the next scheduled check based on the configured interval
		interval.tick().await;

		trace!("Ping update cycle started");

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
			// Create a new database record with the shard ID, latency, and timestamp
			// Then execute the insert operation with proper error context
			let result = PingHistory::insert(ActiveModel {
				shard_id: Set(shard_id.to_string()),
				latency: Set(latency.clone()),
				timestamp: Set(now),
				..Default::default() // Use defaults for any other fields
			})
			.exec(&connection)
			.await
			.context(format!("Failed to insert ping history for shard {} with latency {}", shard_id, latency));

			match result {
				Ok(_) => {
					// Log successful updates at debug level to avoid cluttering logs
					debug!("Updated ping history for shard {}.", shard_id);
				},
				Err(e) => {
					// Log failures at error level for investigation
					error!("Failed to update ping history for shard {}: {:#}", shard_id, e);
					// Continue with other shards despite this error
				},
			}
		}

		trace!("Ping update cycle completed");
	}

	// This is technically unreachable as the loop is infinite,
	// but we return Ok(()) to satisfy the Result return type
	#[allow(unreachable_code)]
	Ok(())
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
	debug!("Game update interval configured for {} seconds", task_intervals.game_update);
	trace!("Initial game data cache state: empty");

	let mut update_count = 0;
	let mut consecutive_failures = 0;
	let mut last_successful_size = 0;

	// Loop indefinitely
	loop {
		// Wait for the next interval tick
		trace!("Waiting for next game data update interval tick");
		interval.tick().await;
		update_count += 1;

		let current_time = chrono::Utc::now();
		info!("Starting game data update cycle #{} at {}", update_count, current_time);

		// Get current cache size for comparison
		let current_size = {
			let apps_read = apps.read().await;
			let size = apps_read.len();
			debug!("Current game data cache size: {} entries", size);
			trace!("Game data cache memory usage estimate: ~{} bytes", size * std::mem::size_of::<(String, u128)>());
			size
		};

 	// Get the game with the provided apps and wait for the result
 	trace!("Calling get_game function with apps cache");
 	match get_game(apps.clone()).await.context("Failed to update game data") {
			Ok(new_entries) => {
				let new_size = apps.read().await.len();
				info!("Game data update cycle #{} completed successfully", update_count);
				debug!("Updated game data cache size: {} entries (added {} new entries)", 
					new_size, new_entries);

				// Calculate and log change statistics
				let size_diff = new_size as i64 - current_size as i64;
				if size_diff != 0 {
					debug!("Game data cache size changed by {} entries", size_diff);

					if size_diff < 0 && size_diff.abs() as usize > (current_size / 2) {
						warn!("Large decrease in game data cache size detected ({}%). This might indicate an API issue.", 
							(size_diff.abs() as f64 / current_size as f64) * 100.0);
					}
				} else {
					trace!("Game data cache size unchanged");
				}

				// Reset failure counter on success
				if consecutive_failures > 0 {
					debug!("Reset consecutive failure counter from {} to 0", consecutive_failures);
					consecutive_failures = 0;
				}

				last_successful_size = new_size;
			},
			Err(e) => {
				consecutive_failures += 1;
				error!("Game data update cycle #{} failed: {}", update_count, e);
				warn!("This is consecutive failure #{} for game data updates", consecutive_failures);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&e));
				debug!("Error context: {}", e.to_string());

				// Provide more context based on error patterns
				if e.to_string().contains("timeout") {
					warn!("API timeout detected. The external service might be experiencing high load or connectivity issues.");
				} else if e.to_string().contains("rate") {
					warn!("Possible rate limiting detected. Consider increasing the update interval.");
				}

				// Suggest action after multiple failures
				if consecutive_failures >= 3 {
					warn!("Multiple consecutive game data update failures detected. Consider checking API availability or credentials.");

					if current_size == 0 && last_successful_size > 0 {
						warn!("Game data cache is empty after previously containing {} entries. This may impact functionality.", 
							last_successful_size);
					}
				}
			}
		}

		debug!("Next game data update scheduled in {} seconds", task_intervals.game_update);
		trace!("Update cycle #{} completed at {}", update_count, chrono::Utc::now());
		trace!("Elapsed time for this cycle: {} ms", 
			(chrono::Utc::now() - current_time).num_milliseconds());
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
	debug!("Activity check interval configured for {} seconds", task_intervals.activity_check);

	let mut update_count = 0;

	// Enter a loop that waits for the next interval tick and spawns a new task to manage the bot's activity
	loop {
		// Wait for the next interval tick
		interval.tick().await;
		update_count += 1;

		info!("Starting activity management cycle #{}", update_count);

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
	debug!("Server image update interval configured for {} seconds", task_intervals.server_image_update);

	let mut update_count = 0;

	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(task_intervals.server_image_update));

	// Loop indefinitely
	loop {
		// Wait for the next interval tick
		interval.tick().await;
		update_count += 1;

		info!("Starting server image management cycle #{}", update_count);

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

	info!("Launching user blacklist update thread");
	debug!("Blacklist update interval configured for {} seconds", task_intervals.blacklisted_user_update);
	trace!("Initial blacklist state: empty");

	let mut update_count = 0;
	let mut consecutive_failures = 0;
	let blacklist_url = "https://raw.githubusercontent.com/ValgulNecron/kasuki/dev/blacklist.json";
	debug!("Blacklist source URL: {}", blacklist_url);

	// Create HTTP client with timeout
	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(10))
		.build()
		.unwrap_or_else(|e| {
			warn!("Failed to create custom HTTP client with timeout: {}", e);
			warn!("Using default reqwest client instead");
			reqwest::Client::new()
		});
	debug!("HTTP client initialized for blacklist updates");

	// Main update loop - runs indefinitely
	loop {
		// Wait for the next scheduled update based on the configured interval
		trace!("Waiting for next blacklist update interval tick");
		interval.tick().await;
		update_count += 1;

		let current_time = chrono::Utc::now();
		info!("Starting blacklist update cycle #{} at {}", update_count, current_time);

		// Get current blacklist size for comparison
		let current_size = {
			let current_blacklist = blacklist_lock.read().await;
			let size = current_blacklist.len();
			debug!("Current blacklist size: {} users", size);

			if size > 0 {
				trace!("First 5 blacklisted users: {:?}", 
					current_blacklist.iter().take(5).cloned().collect::<Vec<String>>());
			}

			size
		};

		// Define the URL where the blacklist is stored
		// Using GitHub raw content ensures we always get the latest version from the specified branch
		info!("Fetching blacklist from remote source");
		trace!("HTTP GET request to: {}", blacklist_url);

		// Fetch the blacklist data from the remote URL with proper error handling
		let blacklist_response = match client.get(blacklist_url).send().await {
			Ok(response) => {
				let status = response.status();
				if status.is_success() {
					debug!("Successfully received blacklist response with status: {}", status);
					trace!("Response headers: {:?}", response.headers());
					response
				} else {
					consecutive_failures += 1;
					error!("Failed to get blacklist: HTTP status {}", status);
					warn!("This is consecutive failure #{} for blacklist updates", consecutive_failures);

					// Provide more context based on status code
					match status.as_u16() {
						404 => warn!("Blacklist file not found. The repository structure might have changed."),
						403 => warn!("Access forbidden. GitHub might be rate limiting the requests."),
						429 => warn!("Too many requests. Definitely being rate limited."),
						500..=599 => warn!("Server error. GitHub might be experiencing issues."),
						_ => debug!("Unexpected status code: {}", status),
					}

					debug!("Next blacklist update scheduled in {} seconds", task_intervals.blacklisted_user_update);
					trace!("Update cycle #{} failed at {}", update_count, chrono::Utc::now());
					continue;
				}
			},
			Err(e) => {
				consecutive_failures += 1;
				error!("Failed to get blacklist: {}", e);
				warn!("This is consecutive failure #{} for blacklist updates", consecutive_failures);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&e));

				// Provide more context based on error type
				if e.is_timeout() {
					warn!("Request timed out. GitHub might be slow or network connectivity issues.");
				} else if e.is_connect() {
					warn!("Connection failed. Check network connectivity.");
				} else if e.is_request() {
					warn!("Request creation failed. This might be a bug in the code.");
				}

				debug!("Next blacklist update scheduled in {} seconds", task_intervals.blacklisted_user_update);
				trace!("Update cycle #{} failed at {}", update_count, chrono::Utc::now());
				continue;
			}
		};

		// Parse the JSON response into a serde_json Value with proper error handling
		trace!("Parsing JSON response from blacklist request");
		let blacklist_json: Value = match blacklist_response.json().await {
			Ok(json) => {
				debug!("Successfully parsed blacklist JSON");
				// Reset failure counter on success
				if consecutive_failures > 0 {
					debug!("Reset consecutive failure counter from {} to 0", consecutive_failures);
					consecutive_failures = 0;
				}

				json
			},
			Err(e) => {
				consecutive_failures += 1;
				error!("Failed to parse blacklist JSON: {}", e);
				warn!("This is consecutive failure #{} for blacklist updates", consecutive_failures);
				debug!("Error type: {}", std::any::type_name_of_val(&e));
				debug!("Next blacklist update scheduled in {} seconds", task_intervals.blacklisted_user_update);
				trace!("Update cycle #{} failed at {}", update_count, chrono::Utc::now());
				continue;
			}
		};

		// Extract user IDs from the JSON array at the "user_id" key
		// This handles the case where the JSON structure might not match our expectations
		trace!("Extracting user_id array from JSON response");
		let user_ids: Vec<String> = match blacklist_json["user_id"].as_array() {
			Some(arr) => {
				debug!("Found user_id array in blacklist with {} entries", arr.len());
				trace!("First 5 user IDs in array: {:?}", 
					arr.iter().take(5).map(|v| v.to_string()).collect::<Vec<String>>());
				arr // If we found an array at the "user_id" key, use it
			},
			None => {
				// If there's no array at "user_id", log an error and skip this update
				consecutive_failures += 1;
				error!("Failed to get user_id array from blacklist JSON");
				warn!("This is consecutive failure #{} for blacklist updates", consecutive_failures);
				debug!("JSON keys available: {:?}", 
					blacklist_json.as_object().map(|o| o.keys().cloned().collect::<Vec<String>>()));
				debug!("Next blacklist update scheduled in {} seconds", task_intervals.blacklisted_user_update);
				trace!("Update cycle #{} failed at {}", update_count, chrono::Utc::now());
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
				error!("Found non-string value in user_id array");
				debug!("Invalid value type: {}", id.to_string());
				"".to_string()
			},
		})
		.filter(|id| !id.is_empty()) // Filter out empty strings
		.collect();

		// Update the shared blacklist with the new data
		info!("Updating blacklist with {} users", user_ids.len());
		trace!("Acquiring write lock on blacklist");

		// This requires acquiring a write lock on the shared data
		let mut blacklist = blacklist_lock.write().await;
		trace!("Write lock acquired");

		// Clear the existing blacklist to remove any users who are no longer blacklisted
		trace!("Clearing existing blacklist with {} entries", blacklist.len());
		blacklist.clear();

		// Shrink the capacity to match the new size
		// This frees memory if the blacklist has significantly decreased in size
		trace!("Shrinking blacklist capacity");
		blacklist.shrink_to_fit();

		// Replace the blacklist with the new list of user IDs
		trace!("Updating blacklist with new user IDs");
		*blacklist = user_ids;

		// Log the result of the update
		let new_size = blacklist.len();
		info!("Blacklist update cycle #{} completed successfully", update_count);

		if new_size != current_size {
			let diff = new_size as i64 - current_size as i64;
			info!("Blacklist size changed: {} → {} users ({:+} users)", 
				current_size, new_size, diff);

			if diff > 10 {
				warn!("Large increase in blacklisted users (+{}). Verify this is intentional.", diff);
			} else if diff < -10 {
				warn!("Large decrease in blacklisted users ({}). Verify this is intentional.", diff);
			}
		} else {
			debug!("Blacklist size unchanged: {} users", new_size);
		}

		debug!("Next blacklist update scheduled in {} seconds", task_intervals.blacklisted_user_update);
		trace!("Update cycle #{} completed at {}", update_count, chrono::Utc::now());
		trace!("Elapsed time for this cycle: {} ms", 
			(chrono::Utc::now() - current_time).num_milliseconds());

		// The lock is automatically released when `blacklist` goes out of scope
		trace!("Write lock released");
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

	info!("Launching bot info update thread");
	debug!("Bot info update interval configured for {} seconds", task_intervals.bot_info);

	let mut update_count = 0;

	loop {
		// Wait for the update interval
		update_interval.tick().await;
		update_count += 1;

		info!("Starting bot info update cycle #{}", update_count);

		// Check if we already have bot info
		let has_existing_info = {
			let current_info = bot_info.read().await;
			current_info.is_some()
		};

		if has_existing_info {
			debug!("Refreshing existing bot information");
		} else {
			info!("Fetching initial bot information");
		}

		// Retrieve the current bot information
		let current_bot_info = match context.http.get_current_application_info().await {
			Ok(info) => {
				debug!("Successfully retrieved bot info for application: {}", info.name);
				info
			},
			Err(e) => {
				error!("Failed to get bot info in cycle #{}: {:?}", update_count, e);
				debug!("Next bot info update scheduled in {} seconds", task_intervals.bot_info);
				continue;
			},
		};

		// Acquire a lock on bot info and update it with the current information
		info!("Updating cached bot information");
		let mut bot_info_lock = bot_info.write().await;

		*bot_info_lock = Some(current_bot_info);
		info!("Bot info update cycle #{} completed successfully", update_count);
		debug!("Next bot info update scheduled in {} seconds", task_intervals.bot_info);
	}
}
