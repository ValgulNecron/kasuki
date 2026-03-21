use crate::structure::steam_game_id_struct::get_game;
use anyhow::Context as AnyhowContext;
use shared::cache::CacheInterface;
use shared::config::TaskIntervalConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, trace, warn};

/// Asynchronously launches the game management thread.
#[tracing::instrument(skip(apps, task_intervals, steam_cache), level = "info")]
pub async fn launch_game_management_thread(
	apps: Arc<RwLock<HashMap<String, u32>>>, task_intervals: TaskIntervalConfig,
	steam_cache: Arc<CacheInterface>,
) {
	let mut interval = interval(Duration::from_secs(task_intervals.game_update));

	info!("Launching the steam management thread!");
	debug!(
		"Game update interval configured for {} seconds",
		task_intervals.game_update
	);
	trace!("Initial game data cache state: empty");

	let mut update_count = 0;
	// Track consecutive failures to escalate warnings after repeated issues
	let mut consecutive_failures = 0;
	// Remember the last known-good cache size to detect data loss after failures
	let mut last_successful_size = 0;

	loop {
		trace!("Waiting for next game data update interval tick");
		interval.tick().await;
		update_count += 1;

		let current_time = chrono::Utc::now();
		info!(
			"Starting game data update cycle #{} at {}",
			update_count, current_time
		);

		let current_size = {
			// Scope the read lock so it's released before the potentially slow API call
			let apps_read = apps.read().await;
			let size = apps_read.len();
			debug!("Current game data cache size: {} entries", size);
			trace!(
				"Game data cache memory usage estimate: ~{} bytes",
				size * std::mem::size_of::<(String, u32)>()
			);
			size
		};

		trace!("Calling get_game function with apps cache");
		match get_game(apps.clone(), steam_cache.clone())
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

				let size_diff = new_size as i64 - current_size as i64;
				if size_diff != 0 {
					debug!("Game data cache size changed by {} entries", size_diff);

					// A >50% drop likely means the API returned partial data, not a real removal
					if size_diff < 0 && size_diff.abs() as usize > (current_size / 2) {
						warn!(
							"Large decrease in game data cache size detected ({}%). This might indicate an API issue.",
							(size_diff.abs() as f64 / current_size as f64) * 100.0
						);
					}
				} else {
					trace!("Game data cache size unchanged");
				}

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

				debug!("Error type: {}", std::any::type_name_of_val(&e));
				debug!("Error context: {}", e.to_string());

				// Inspect the error message to classify the failure type for targeted warnings
				if e.to_string().contains("timeout") {
					warn!(
						"API timeout detected. The external service might be experiencing high load or connectivity issues."
					);
				} else if e.to_string().contains("rate") {
					warn!(
						"Possible rate limiting detected. Consider increasing the update interval."
					);
				}

				// 3 failures in a row suggests a persistent issue, not a transient glitch
				if consecutive_failures >= 3 {
					warn!(
						"Multiple consecutive game data update failures detected. Consider checking API availability or credentials."
					);

					// Cache went from populated to empty — commands relying on game data will break
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
