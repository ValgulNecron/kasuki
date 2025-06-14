use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender;
use moka::future::Cache;
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait};
use serde_json::Value;
use serenity::all::{Context as SerenityContext, CurrentApplicationInfo, ShardId};
use serenity::gateway::{ShardRunnerInfo, ShardRunnerMessage};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
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
use crate::structure::steam_game_id_struct::get_game;
use anyhow::{Context as AnyhowContext, Result};
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
	debug!(
		"Task intervals configuration: anisong_update={}s, random_stats_update={}s, activity_check={}s, game_update={}s, ping_update={}s, bot_info_update={}s, blacklisted_user_update={}s, server_image_update={}s, before_server_image={}s",
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
	debug!(
		"Spawning anisong database update task (interval: {}s)",
		task_intervals.anisong_update
	);
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
	debug!(
		"Spawning random stats update task (interval: {}s)",
		task_intervals.random_stats_update
	);
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
	debug!(
		"Spawning activity management task (interval: {}s)",
		task_intervals.activity_check
	);
	let task_intervals_c = task_intervals.clone();
	let ctx_c = ctx.clone();
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
	debug!(
		"Spawning game management task (interval: {}s)",
		task_intervals.game_update
	);
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
	debug!(
		"Spawning ping manager task (interval: {}s)",
		task_intervals.ping_update
	);
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

	// === SECURITY TASKS ===
	info!("Launching security background tasks");

	// Launch user blacklist update task
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

	// === VISUAL TASKS (with delay) ===
	info!(
		"Scheduling visual tasks with delay of {}s",
		task_intervals.before_server_image
	);

	// Wait before launching server image management
	// This delay ensures other critical tasks are running before starting image generation
	debug!(
		"Waiting {}s before launching server image management task",
		task_intervals.before_server_image
	);
	sleep(Duration::from_secs(task_intervals.before_server_image)).await;

	let image_config = bot_data.config.image.clone();

	// Launch server image management thread
	debug!(
		"Spawning server image management task (interval: {}s)",
		task_intervals.server_image_update
	);
	let task_intervals_c = task_intervals.clone();
	let ctx_c = ctx.clone();
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
	debug!(
		"Registered {} background tasks with shutdown handlers",
		shutdown_receivers.len()
	);

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
	debug!(
		"Anisong database will update every {} seconds",
		task_intervals.anisong_update
	);

	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.anisong_update));
	let mut update_count = 0;
	let mut consecutive_failures = 0;
	let mut in_recovery_mode = false;
	let max_backoff_seconds = 3600; // Maximum backoff of 1 hour
	let base_interval = task_intervals.anisong_update;

	loop {
		// Wait for the next scheduled update time
		trace!("Waiting for next anisong update interval tick");
		interval.tick().await;
		update_count += 1;

		info!("Starting anisong database update cycle #{}", update_count);
		debug!("Current time: {}", chrono::Utc::now());

		// Perform a database health check before attempting the update
		let db_health_check = check_database_health(&db).await;
		if !db_health_check {
			consecutive_failures += 1;
			error!(
				"Database health check failed before anisong update cycle #{}",
				update_count
			);
			warn!(
				"This is consecutive failure #{} for anisong updates",
				consecutive_failures
			);

			// Enter recovery mode if not already in it
			if !in_recovery_mode && consecutive_failures >= 2 {
				warn!(
					"Entering recovery mode for anisong updates due to database health check failures"
				);
				in_recovery_mode = true;
			}

			continue; // Skip this update cycle
		}

		// Attempt to update the anisong database
		trace!("Calling get_anisong function with database connection");
		match get_anisong(db.clone()).await {
			Ok(count) => {
				info!(
					"Anisong database update cycle #{} completed successfully",
					update_count
				);
				debug!("Updated {} anisong records in the database", count);

				// Reset failure counter and recovery mode on success
				if consecutive_failures > 0 || in_recovery_mode {
					info!(
						"Anisong update recovered after {} consecutive failures",
						consecutive_failures
					);
					consecutive_failures = 0;

					if in_recovery_mode {
						info!("Exiting recovery mode for anisong updates");
						in_recovery_mode = false;
						// Reset to normal interval
						interval = tokio::time::interval(Duration::from_secs(base_interval));
					}
				}
			},
			Err(err) => {
				consecutive_failures += 1;

				// Categorize the error for better handling
				// This helps us apply different recovery strategies based on the error type
				// For example, network errors might be transient and benefit from quick retries,
				// while database errors might require more careful handling
				let error_category = categorize_error(&err);

				error!(
					"Anisong database update cycle #{} failed ({}): {:#}",
					update_count, error_category, err
				);
				warn!(
					"This is consecutive failure #{} for anisong updates",
					consecutive_failures
				);

				// Log more details about the error for debugging
				// The type_name helps identify the exact error type for troubleshooting
				debug!("Error type: {}", std::any::type_name_of_val(&err));
				// The error chain shows the full context trail, which is valuable for nested errors
				debug!("Error context chain: {:?}", err.chain().collect::<Vec<_>>());

				// Apply different recovery strategies based on error category
				// Each error type requires a different approach to recovery:
				match error_category {
					ErrorCategory::Network => {
						// Network errors (connection issues, timeouts) are often transient
						// We use exponential backoff to avoid overwhelming the network
						warn!(
							"Network error detected in anisong update. Will retry with exponential backoff."
						);

						// Enter recovery mode after 2 consecutive network failures
						// This threshold is lower than for "Other" errors because network issues
						// are more likely to be temporary but need time to resolve
						if !in_recovery_mode && consecutive_failures >= 2 {
							warn!(
								"Entering recovery mode with exponential backoff for anisong updates"
							);
							in_recovery_mode = true;
						}
					},
					ErrorCategory::Database => {
						// Database errors might indicate more serious issues that need attention
						// We perform an explicit test to check if the database is still accessible
						error!(
							"Database error detected in anisong update. Attempting database reconnection test."
						);

						// Test if we can still connect to the database with a simple query
						// This helps distinguish between temporary glitches and serious connection issues
						if let Err(e) = test_database_connection(&db).await {
							error!("Database reconnection test failed: {:#}", e);

							// If the test fails, we definitely need recovery mode
							// Database issues often require more time to resolve or manual intervention
							if !in_recovery_mode {
								warn!(
									"Entering recovery mode with exponential backoff for anisong updates due to database errors"
								);
								in_recovery_mode = true;
							}
						}
						// Note: If the test succeeds, we don't enter recovery mode yet,
						// as it might have been a transient database issue
					},
					ErrorCategory::Api => {
						// API errors (rate limiting, service unavailable) need careful handling
						// These often benefit from backing off to respect API limits
						warn!(
							"API error detected in anisong update. Will retry with exponential backoff."
						);

						// Similar threshold to network errors, as API issues are often temporary
						// but need time to resolve (e.g., rate limits need time to reset)
						if !in_recovery_mode && consecutive_failures >= 2 {
							warn!(
								"Entering recovery mode with exponential backoff for anisong updates"
							);
							in_recovery_mode = true;
						}
					},
					ErrorCategory::Other => {
						// Unknown errors are harder to categorize and might be more serious
						// We're more cautious about entering recovery mode for these
						warn!("Unknown error detected in anisong update. Will retry normally.");

						// Higher threshold (3 failures) before entering recovery mode
						// This gives more chances for transient issues to resolve themselves
						// without aggressive backoff, while still protecting against persistent problems
						if !in_recovery_mode && consecutive_failures >= 3 {
							warn!(
								"Entering recovery mode for anisong updates due to persistent unknown errors"
							);
							in_recovery_mode = true;
						}
					},
				}

				// Additional actions for persistent failures
				if consecutive_failures >= 5 {
					error!(
						"Critical: Anisong update has failed {} consecutive times. Manual intervention may be required.",
						consecutive_failures
					);
				}
			},
		}

		debug!(
			"Next anisong database update scheduled in {} seconds",
			if in_recovery_mode {
				let backoff_factor = 2u64.pow(consecutive_failures.min(10) as u32);
				(base_interval * backoff_factor).min(max_backoff_seconds)
			} else {
				base_interval
			}
		);
		trace!(
			"Update cycle #{} completed at {}",
			update_count,
			chrono::Utc::now()
		);
	}
}

/// Categorizes an error to determine the appropriate recovery strategy
///
/// This function analyzes the error message to classify errors into distinct categories,
/// which allows for tailored recovery strategies based on the error type.
///
/// # Error Classification Logic
///
/// The function uses a simple but effective string matching approach to categorize errors:
/// 1. It converts the error to a string representation that includes the full context
/// 2. It checks for specific keywords that indicate different error categories
/// 3. It returns the most specific category that matches
///
/// # Categories and Their Significance
///
/// * `Network` - Connection issues, timeouts, or other network-related problems
///   These often benefit from retries with backoff as they may be transient
///
/// * `Database` - SQL errors, query failures, or database connection issues
///   These might require special handling like connection testing or admin notification
///
/// * `API` - Issues with external API calls, status codes, or response parsing
///   These often need backoff strategies to respect rate limits or service availability
///
/// * `Other` - Any errors that don't fit the above categories
///   These get a more conservative recovery approach as their nature is less predictable
///
/// # Implementation Note
///
/// This approach has limitations - it relies on error messages containing certain keywords,
/// which may change if error formatting changes. A more robust approach would be to use
/// error types or error codes, but this would require changes to the error handling
/// throughout the codebase.
fn categorize_error(err: &anyhow::Error) -> ErrorCategory {
	// Convert the error to a string that includes the full context chain
	// The {:#} format specifier ensures we get the alternate (verbose) representation
	let error_string = format!("{:#}", err);

	// Check for network-related errors
	// These keywords commonly appear in network failure scenarios
	if error_string.contains("connection")
		|| error_string.contains("timeout")
		|| error_string.contains("network")
		|| error_string.contains("connect")
	{
		// Network errors are often transient and benefit from retries with backoff
		return ErrorCategory::Network;
	}

	// Check for database-related errors
	// These keywords commonly appear in database failure scenarios
	if error_string.contains("database")
		|| error_string.contains("sql")
		|| error_string.contains("query")
		|| error_string.contains("db")
	{
		// Database errors might indicate more serious issues that need attention
		return ErrorCategory::Database;
	}

	// Check for API-related errors
	// These keywords commonly appear in API failure scenarios
	if error_string.contains("api")
		|| error_string.contains("status")
		|| error_string.contains("response")
		|| error_string.contains("request")
	{
		// API errors often relate to rate limiting or service availability
		return ErrorCategory::Api;
	}

	// If none of the above patterns match, we classify as "Other"
	// This is a catch-all category for errors we couldn't specifically identify
	// We use a more conservative recovery approach for these unknown errors
	ErrorCategory::Other
}

/// Enum representing different categories of errors for targeted recovery strategies
///
/// This enum defines the possible error categories that can be identified by the
/// `categorize_error` function. Each category represents a distinct class of errors
/// that may require different recovery approaches.
///
/// # Categories
///
/// * `Network` - Errors related to network connectivity, such as connection failures,
///   timeouts, or DNS resolution issues. These errors are often transient and can
///   benefit from retry strategies with exponential backoff.
///
/// * `Database` - Errors related to database operations, such as connection failures,
///   query errors, or constraint violations. These may require special handling like
///   connection testing or administrator notification.
///
/// * `Api` - Errors related to external API interactions, such as rate limiting,
///   authentication failures, or service unavailability. These often need careful
///   backoff strategies to respect API limits.
///
/// * `Other` - A catch-all category for errors that don't fit into the above categories.
///   These get a more conservative recovery approach as their nature is less predictable.
///
/// # Usage
///
/// The error category is used to determine the appropriate recovery strategy in the
/// error handling code. Different categories may have different thresholds for entering
/// recovery mode or different backoff strategies.
#[derive(Debug, PartialEq)]
enum ErrorCategory {
	/// Network-related errors (connection issues, timeouts)
	Network,
	/// Database-related errors (SQL errors, connection failures)
	Database,
	/// API-related errors (rate limiting, service unavailability)
	Api,
	/// Errors that don't fit into other categories
	Other,
}

/// Implements the Display trait for ErrorCategory to provide string representations
///
/// This implementation allows ErrorCategory values to be easily included in formatted strings,
/// log messages, and error reports. Each category is represented by a lowercase string
/// that clearly identifies the error type.
///
/// # Usage
///
/// This is particularly useful in logging contexts where we want to include the error category
/// in log messages. For example:
///
/// ```
/// error!("Task failed ({}): {}", error_category, error_message);
/// ```
///
/// The string representations are intentionally kept simple and lowercase to maintain
/// consistency in log formatting and to make log parsing easier.
impl Display for ErrorCategory {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		// Convert each enum variant to a simple, lowercase string representation
		// These strings are used in log messages and error reports
		match self {
			ErrorCategory::Network => {
				// Network errors are represented as "network"
				write!(f, "network")
			},
			ErrorCategory::Database => {
				// Database errors are represented as "database"
				write!(f, "database")
			},
			ErrorCategory::Api => {
				// API errors are represented as "api"
				write!(f, "api")
			},
			ErrorCategory::Other => {
				// Other/unknown errors are represented as "other"
				write!(f, "other")
			},
		}
	}
}

/// Performs a basic health check on the database connection
async fn check_database_health(db: &Arc<DatabaseConnection>) -> bool {
	trace!("Performing database health check");

	// Try a simple query to check if the database is responsive
	match db.execute_unprepared("SELECT 1").await {
		Ok(_) => {
			trace!("Database health check passed");
			true
		},
		Err(e) => {
			error!("Database health check failed: {:#}", e);
			false
		},
	}
}

/// Tests the database connection by attempting a simple query
async fn test_database_connection(db: &Arc<DatabaseConnection>) -> Result<()> {
	debug!("Testing database connection");

	db.execute_unprepared("SELECT 1")
		.await
		.context("Failed to execute test query during database connection test")?;

	debug!("Database connection test successful");
	Ok(())
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
async fn ping_manager_thread(
	ctx: SerenityContext, db_config: DbConfig, task_intervals: TaskIntervalConfig,
) -> Result<()> {
	// Log the initialization of the ping monitoring thread
	info!("Launching the ping monitoring thread!");
	debug!(
		"Ping update interval configured for {} seconds",
		task_intervals.ping_update
	);
	trace!(
		"Initializing ping monitoring with database config: {:?}",
		db_config
	);

	// Retrieve the shard manager from the bot's context
	// The shard manager contains information about all active shards
	trace!("Attempting to retrieve shard manager from context");
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
		Some(shard_manager) => {
			debug!("Successfully retrieved shard manager");
			shard_manager
		},
		None => {
			// If the shard manager is not available (which might happen during startup),
			// sleep for the configured interval and then retry by recursively calling this function
			warn!(
				"Shard manager not available, waiting for {} seconds before retry",
				task_intervals.ping_update
			);
			tokio::time::sleep(Duration::from_secs(task_intervals.ping_update)).await;
			debug!("Retrying shard manager retrieval");
			Box::pin(ping_manager_thread(ctx, db_config, task_intervals)).await?;
			return Ok(());
		},
	};

	// Set up a periodic interval for checking shard latency
	// This determines how frequently we'll record ping data
	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.ping_update));
	debug!(
		"Set up ping check interval timer for {} seconds",
		task_intervals.ping_update
	);

	// Establish a connection to the database for recording ping history
	// This connection is reused for all database operations to avoid repeatedly connecting
	info!("Establishing database connection for ping history recording");
	trace!(
		"Using database URL from config: {}",
		get_url(db_config.clone())
	);

	let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
		Ok(conn) => {
			info!("Successfully connected to database for ping history");
			conn
		},
		Err(e) => {
			error!("Failed to connect to database: {:#}", e);
			return Err(e).context(format!(
				"Failed to connect to database with config: {:?}",
				db_config
			));
		},
	};

	// Main monitoring loop - runs indefinitely
	let mut cycle_count = 0;
	let mut total_shards_processed = 0;
	let mut successful_updates = 0;
	let mut failed_updates = 0;

	loop {
		// Wait for the next scheduled check based on the configured interval
		interval.tick().await;
		cycle_count += 1;

		let current_time = chrono::Utc::now();
		info!(
			"Starting ping update cycle #{} at {}",
			cycle_count, current_time
		);
		trace!("Ping update cycle started");

		// Get a reference to the shard manager
		// DashMap provides thread-safe concurrent access without explicit locking
		let runner = &shard_manager;
		let shard_count = runner.len();
		debug!("Processing {} shards in this cycle", shard_count);

		// Reset counters for this cycle
		let mut cycle_successful_updates = 0;
		let mut cycle_failed_updates = 0;

		// Iterate through each shard to record its latency
		for entry in runner.iter() {
			// Get the shard ID (a numeric identifier for the shard)
			let shard_id = entry.key();
			trace!("Processing shard {}", shard_id);

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

				trace!("Shard {} current latency: {}ms", shard_id, latency);

				// Get the current UTC timestamp for recording when this measurement was taken
				let now = chrono::Utc::now().naive_utc();

				(now, latency)
			};

			// Create a new database record with the shard ID, latency, and timestamp
			// Then execute the insert operation
			// Create a new database record with the shard ID, latency, and timestamp
			// Then execute the insert operation with proper error context
			trace!(
				"Inserting ping history record for shard {} with latency {}",
				shard_id, latency
			);
			let result = PingHistory::insert(ActiveModel {
				shard_id: Set(shard_id.to_string()),
				latency: Set(latency.clone()),
				timestamp: Set(now),
				..Default::default() // Use defaults for any other fields
			})
			.exec(&connection)
			.await
			.context(format!(
				"Failed to insert ping history for shard {} with latency {}",
				shard_id, latency
			));

			match result {
				Ok(_) => {
					// Log successful updates at debug level to avoid cluttering logs
					debug!("Updated ping history for shard {}.", shard_id);
					cycle_successful_updates += 1;
					successful_updates += 1;
					total_shards_processed += 1;
				},
				Err(e) => {
					// Log failures at error level for investigation
					error!(
						"Failed to update ping history for shard {}: {:#}",
						shard_id, e
					);
					cycle_failed_updates += 1;
					failed_updates += 1;
					total_shards_processed += 1;
					// Continue with other shards despite this error
				},
			}
		}

		// Log summary of this cycle
		let cycle_end_time = chrono::Utc::now();
		let cycle_duration = cycle_end_time.signed_duration_since(current_time);

		info!(
			"Ping update cycle #{} completed: processed {} shards ({} successful, {} failed) in {:?}",
			cycle_count,
			shard_count,
			cycle_successful_updates,
			cycle_failed_updates,
			cycle_duration
		);

		debug!(
			"Ping monitoring stats: total processed: {}, success rate: {:.2}%",
			total_shards_processed,
			if total_shards_processed > 0 {
				(successful_updates as f64 / total_shards_processed as f64) * 100.0
			} else {
				0.0
			}
		);

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

async fn launch_game_management_thread(
	apps: Arc<RwLock<HashMap<String, u128>>>, task_intervals: TaskIntervalConfig,
) {
	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(task_intervals.game_update));

	// Log a message indicating that the steam management thread is being launched
	info!("Launching the steam management thread!");
	debug!(
		"Game update interval configured for {} seconds",
		task_intervals.game_update
	);
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
		info!(
			"Starting game data update cycle #{} at {}",
			update_count, current_time
		);

		// Get current cache size for comparison
		let current_size = {
			let apps_read = apps.read().await;
			let size = apps_read.len();
			debug!("Current game data cache size: {} entries", size);
			trace!(
				"Game data cache memory usage estimate: ~{} bytes",
				size * std::mem::size_of::<(String, u128)>()
			);
			size
		};

		// Get the game with the provided apps and wait for the result
		trace!("Calling get_game function with apps cache");
		match get_game(apps.clone())
			.await
			.context("Failed to update game data")
		{
			Ok(new_entries) => {
				let new_size = apps.read().await.len();
				info!(
					"Game data update cycle #{} completed successfully",
					update_count
				);
				debug!(
					"Updated game data cache size: {} entries (added {} new entries)",
					new_size, new_entries
				);

				// Calculate and log change statistics
				let size_diff = new_size as i64 - current_size as i64;
				if size_diff != 0 {
					debug!("Game data cache size changed by {} entries", size_diff);

					if size_diff < 0 && size_diff.abs() as usize > (current_size / 2) {
						warn!(
							"Large decrease in game data cache size detected ({}%). This might indicate an API issue.",
							(size_diff.abs() as f64 / current_size as f64) * 100.0
						);
					}
				} else {
					trace!("Game data cache size unchanged");
				}

				// Reset failure counter on success
				if consecutive_failures > 0 {
					debug!(
						"Reset consecutive failure counter from {} to 0",
						consecutive_failures
					);
					consecutive_failures = 0;
				}

				last_successful_size = new_size;
			},
			Err(e) => {
				consecutive_failures += 1;
				error!("Game data update cycle #{} failed: {}", update_count, e);
				warn!(
					"This is consecutive failure #{} for game data updates",
					consecutive_failures
				);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&e));
				debug!("Error context: {}", e.to_string());

				// Provide more context based on error patterns
				if e.to_string().contains("timeout") {
					warn!(
						"API timeout detected. The external service might be experiencing high load or connectivity issues."
					);
				} else if e.to_string().contains("rate") {
					warn!(
						"Possible rate limiting detected. Consider increasing the update interval."
					);
				}

				// Suggest action after multiple failures
				if consecutive_failures >= 3 {
					warn!(
						"Multiple consecutive game data update failures detected. Consider checking API availability or credentials."
					);

					if current_size == 0 && last_successful_size > 0 {
						warn!(
							"Game data cache is empty after previously containing {} entries. This may impact functionality.",
							last_successful_size
						);
					}
				}
			},
		}

		debug!(
			"Next game data update scheduled in {} seconds",
			task_intervals.game_update
		);
		trace!(
			"Update cycle #{} completed at {}",
			update_count,
			chrono::Utc::now()
		);
		trace!(
			"Elapsed time for this cycle: {} ms",
			(chrono::Utc::now() - current_time).num_milliseconds()
		);
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
	debug!(
		"Activity check interval configured for {} seconds",
		task_intervals.activity_check
	);

	let mut update_count = 0;
	let mut consecutive_failures = 0;
	let mut in_recovery_mode = false;
	let max_backoff_seconds = 3600; // Maximum backoff of 1 hour
	let base_interval = task_intervals.activity_check;

	// Enter a loop that waits for the next interval tick and spawns a new task to manage the bot's activity
	loop {
		// Wait for the next interval tick
		trace!("Waiting for next activity management interval tick");
		interval.tick().await;
		update_count += 1;

		info!("Starting activity management cycle #{}", update_count);
		debug!("Current time: {}", chrono::Utc::now());

		// Clone the context and db_type arguments
		let ctx = ctx.clone();
		let anilist_cache_clone = anilist_cache.clone();
		let db_config_clone = db_config.clone();

		// Spawn a new task to manage the bot's activity and handle errors
		tokio::spawn(manage_activity(ctx, anilist_cache_clone, db_config_clone));

		debug!(
			"Next activity management cycle scheduled in {} seconds",
			if in_recovery_mode {
				let backoff_factor = 2u64.pow(consecutive_failures.min(10) as u32);
				(base_interval * backoff_factor).min(max_backoff_seconds)
			} else {
				base_interval
			}
		);
		trace!(
			"Update cycle #{} completed at {}",
			update_count,
			chrono::Utc::now()
		);
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
	debug!(
		"Server image update interval configured for {} seconds",
		task_intervals.server_image_update
	);

	let mut update_count = 0;
	let mut consecutive_failures = 0;
	let mut in_recovery_mode = false;
	let max_backoff_seconds = 3600; // Maximum backoff of 1 hour
	let base_interval = task_intervals.server_image_update;

	// Create an interval for periodic updates
	let mut interval = interval(Duration::from_secs(task_intervals.server_image_update));

	// Loop indefinitely
	loop {
		// Wait for the next interval tick
		trace!("Waiting for next server image management interval tick");
		interval.tick().await;
		update_count += 1;

		// If we're in recovery mode, apply exponential backoff
		let current_interval = if in_recovery_mode {
			let backoff_factor = 2u64.pow(consecutive_failures.min(10) as u32); // Prevent overflow
			let backoff_seconds = (base_interval * backoff_factor).min(max_backoff_seconds);

			info!(
				"Server image management in recovery mode: using extended interval of {} seconds",
				backoff_seconds
			);

			// Reset the interval with the new backoff duration
			interval = tokio::time::interval(Duration::from_secs(backoff_seconds));
			backoff_seconds
		} else {
			base_interval
		};

		info!("Starting server image management cycle #{}", update_count);
		debug!("Current time: {}", chrono::Utc::now());

		// Create a task to handle server image management with error handling
		let ctx_clone = ctx.clone();
		let image_config_clone = image_config.clone();
		let connection_clone = connection.clone();

		// Wrap the server_image_management call in a task to catch any panics
		let server_image_handle = tokio::spawn(async move {
			// Use a timeout to prevent the task from running indefinitely
			match tokio::time::timeout(
				Duration::from_secs(300), // 5 minute timeout
				server_image_management(&ctx_clone, image_config_clone, connection_clone),
			)
			.await
			{
				Ok(result) => Ok(result),
				Err(e) => Err(anyhow::anyhow!(
					"Server image management task timed out after 300 seconds: {}",
					e
				)),
			}
		});

		// Wait for the task to complete and check for errors
		match server_image_handle.await {
			Ok(Ok(_)) => {
				info!(
					"Server image management cycle #{} completed successfully",
					update_count
				);

				// Reset failure counter and recovery mode on success
				if consecutive_failures > 0 || in_recovery_mode {
					info!(
						"Server image management recovered after {} consecutive failures",
						consecutive_failures
					);
					consecutive_failures = 0;

					if in_recovery_mode {
						info!("Exiting recovery mode for server image management");
						in_recovery_mode = false;
						// Reset to normal interval
						interval = tokio::time::interval(Duration::from_secs(base_interval));
					}
				}
			},
			Ok(Err(e)) => {
				consecutive_failures += 1;
				error!(
					"Server image management cycle #{} failed: {:#}",
					update_count, e
				);
				warn!(
					"This is consecutive failure #{} for server image management",
					consecutive_failures
				);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&e));

				if let Some(err) = e.downcast_ref::<anyhow::Error>() {
					debug!("Error context chain: {:?}", err.chain().collect::<Vec<_>>());
				}

				// Enter recovery mode after multiple consecutive failures
				if !in_recovery_mode && consecutive_failures >= 3 {
					warn!(
						"Entering recovery mode with exponential backoff for server image management"
					);
					in_recovery_mode = true;
				}

				// Additional actions for persistent failures
				if consecutive_failures >= 5 {
					error!(
						"Critical: Server image management has failed {} consecutive times. Manual intervention may be required.",
						consecutive_failures
					);
				}
			},
			Err(e) => {
				consecutive_failures += 1;
				error!(
					"Server image management task #{} panicked: {:#}",
					update_count, e
				);
				warn!(
					"This is consecutive failure #{} for server image management",
					consecutive_failures
				);

				// Enter recovery mode after task panics
				if !in_recovery_mode && consecutive_failures >= 2 {
					warn!(
						"Entering recovery mode with exponential backoff for server image management due to task panic"
					);
					in_recovery_mode = true;
				}
			},
		}

		debug!(
			"Next server image management cycle scheduled in {} seconds",
			if in_recovery_mode {
				let backoff_factor = 2u64.pow(consecutive_failures.min(10) as u32);
				(base_interval * backoff_factor).min(max_backoff_seconds)
			} else {
				base_interval
			}
		);
		trace!(
			"Update cycle #{} completed at {}",
			update_count,
			chrono::Utc::now()
		);
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
async fn update_user_blacklist(
	blacklist_lock: Arc<RwLock<Vec<String>>>, task_intervals: TaskIntervalConfig,
) {
	// Set up a periodic interval for blacklist updates
	// This determines how frequently we'll check for changes to the blacklist
	let mut interval =
		tokio::time::interval(Duration::from_secs(task_intervals.blacklisted_user_update));

	info!("Launching user blacklist update thread");
	debug!(
		"Blacklist update interval configured for {} seconds",
		task_intervals.blacklisted_user_update
	);
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
		info!(
			"Starting blacklist update cycle #{} at {}",
			update_count, current_time
		);

		// Get current blacklist size for comparison
		let current_size = {
			let current_blacklist = blacklist_lock.read().await;
			let size = current_blacklist.len();
			debug!("Current blacklist size: {} users", size);

			if size > 0 {
				trace!(
					"First 5 blacklisted users: {:?}",
					current_blacklist
						.iter()
						.take(5)
						.cloned()
						.collect::<Vec<String>>()
				);
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
					debug!(
						"Successfully received blacklist response with status: {}",
						status
					);
					trace!("Response headers: {:?}", response.headers());
					response
				} else {
					consecutive_failures += 1;
					error!("Failed to get blacklist: HTTP status {}", status);
					warn!(
						"This is consecutive failure #{} for blacklist updates",
						consecutive_failures
					);

					// Provide more context based on status code
					match status.as_u16() {
						404 => warn!(
							"Blacklist file not found. The repository structure might have changed."
						),
						403 => {
							warn!("Access forbidden. GitHub might be rate limiting the requests.")
						},
						429 => warn!("Too many requests. Definitely being rate limited."),
						500..=599 => warn!("Server error. GitHub might be experiencing issues."),
						_ => debug!("Unexpected status code: {}", status),
					}

					debug!(
						"Next blacklist update scheduled in {} seconds",
						task_intervals.blacklisted_user_update
					);
					trace!(
						"Update cycle #{} failed at {}",
						update_count,
						chrono::Utc::now()
					);
					continue;
				}
			},
			Err(e) => {
				consecutive_failures += 1;
				error!("Failed to get blacklist: {}", e);
				warn!(
					"This is consecutive failure #{} for blacklist updates",
					consecutive_failures
				);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&e));

				// Provide more context based on error type
				if e.is_timeout() {
					warn!(
						"Request timed out. GitHub might be slow or network connectivity issues."
					);
				} else if e.is_connect() {
					warn!("Connection failed. Check network connectivity.");
				} else if e.is_request() {
					warn!("Request creation failed. This might be a bug in the code.");
				}

				debug!(
					"Next blacklist update scheduled in {} seconds",
					task_intervals.blacklisted_user_update
				);
				trace!(
					"Update cycle #{} failed at {}",
					update_count,
					chrono::Utc::now()
				);
				continue;
			},
		};

		// Parse the JSON response into a serde_json Value with proper error handling
		trace!("Parsing JSON response from blacklist request");
		let blacklist_json: Value = match blacklist_response.json().await {
			Ok(json) => {
				debug!("Successfully parsed blacklist JSON");
				// Reset failure counter on success
				if consecutive_failures > 0 {
					debug!(
						"Reset consecutive failure counter from {} to 0",
						consecutive_failures
					);
					consecutive_failures = 0;
				}

				json
			},
			Err(e) => {
				consecutive_failures += 1;
				error!("Failed to parse blacklist JSON: {}", e);
				warn!(
					"This is consecutive failure #{} for blacklist updates",
					consecutive_failures
				);
				debug!("Error type: {}", std::any::type_name_of_val(&e));
				debug!(
					"Next blacklist update scheduled in {} seconds",
					task_intervals.blacklisted_user_update
				);
				trace!(
					"Update cycle #{} failed at {}",
					update_count,
					chrono::Utc::now()
				);
				continue;
			},
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
            }
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
            }
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
                }
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
		trace!(
			"Clearing existing blacklist with {} entries",
			blacklist.len()
		);
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
		info!(
			"Blacklist update cycle #{} completed successfully",
			update_count
		);

		if new_size != current_size {
			let diff = new_size as i64 - current_size as i64;
			info!(
				"Blacklist size changed: {} → {} users ({:+} users)",
				current_size, new_size, diff
			);

			if diff > 10 {
				warn!(
					"Large increase in blacklisted users (+{}). Verify this is intentional.",
					diff
				);
			} else if diff < -10 {
				warn!(
					"Large decrease in blacklisted users ({}). Verify this is intentional.",
					diff
				);
			}
		} else {
			debug!("Blacklist size unchanged: {} users", new_size);
		}

		debug!(
			"Next blacklist update scheduled in {} seconds",
			task_intervals.blacklisted_user_update
		);
		trace!(
			"Update cycle #{} completed at {}",
			update_count,
			chrono::Utc::now()
		);
		trace!(
			"Elapsed time for this cycle: {} ms",
			(chrono::Utc::now() - current_time).num_milliseconds()
		);

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
