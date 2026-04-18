use crate::structure::steam_game_id_struct::get_game;
use crate::structure::steam_game_index::SteamGameIndex;
use anyhow::Context as AnyhowContext;
use arc_swap::ArcSwap;
use shared::cache::CacheInterface;
use shared::config::TaskIntervalConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Asynchronously launches the game management thread.
#[tracing::instrument(skip(apps, task_intervals, steam_cache), level = "info")]
pub async fn launch_game_management_thread(
	apps: Arc<ArcSwap<SteamGameIndex>>, task_intervals: TaskIntervalConfig,
	steam_cache: Arc<CacheInterface>,
) {
	let mut interval = interval(Duration::from_secs(task_intervals.game_update));

	info!("Launching the steam management thread!");

	let mut update_count = 0;
	let mut consecutive_failures = 0;
	let mut last_successful_size = 0;

	loop {
		interval.tick().await;
		update_count += 1;

		let current_time = chrono::Utc::now();
		info!(
			"Starting game data update cycle #{} at {}",
			update_count, current_time
		);

		let current_size = apps.load().len();

		match get_game(apps.clone(), steam_cache.clone())
			.await
			.context("Failed to update game data")
		{
			Ok(new_entries) => {
				let new_size = apps.load().len();
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

					if size_diff < 0 && size_diff.abs() as usize > (current_size / 2) {
						warn!(
							"Large decrease in game data cache size detected ({}%). This might indicate an API issue.",
							(size_diff.abs() as f64 / current_size as f64) * 100.0
						);
					}
				}

				if consecutive_failures > 0 {
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

				if e.to_string().contains("timeout") {
					warn!(
						"API timeout detected. The external service might be experiencing high load or connectivity issues."
					);
				} else if e.to_string().contains("rate") {
					warn!(
						"Possible rate limiting detected. Consider increasing the update interval."
					);
				}

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

	}
}
