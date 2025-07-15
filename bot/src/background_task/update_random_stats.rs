use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, trace, warn};

use crate::config::TaskIntervalConfig;
use crate::constant::RANDOM_STATS_PATH;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::site_statistic_anime::{AnimeStat, AnimeStatVariables};
use crate::structure::run::anilist::site_statistic_manga::{MangaStat, MangaStatVariables};

/// Represents the random statistics of anime and manga.
#[derive(Debug, Deserialize, Clone, Serialize)]

pub struct RandomStat {
	/// The last page of anime statistics.
	pub anime_last_page: i32,
	/// The last page of manga statistics.
	pub manga_last_page: i32,
}

impl Default for RandomStat {
	/// Returns a default `RandomStat` with `anime_last_page` set to 1796 and `manga_last_page` set to 1796.

	fn default() -> Self {
		Self {
			anime_last_page: 1796,
			manga_last_page: 1796,
		}
	}
}

/// Launches a background task to update the random statistics at regular intervals.
///
/// # Arguments
///
/// * `anilist_cache` - A cache for storing Anilist API responses.
/// * `task_intervals` - Configuration for task intervals.

pub async fn update_random_stats_launcher(
	anilist_cache: Arc<RwLock<Cache<String, String>>>, task_intervals: TaskIntervalConfig,
) {
	// Log the start of the random stats update task.
	info!("Launching random stats update background task");
	debug!(
		"Random stats update interval configured for {} seconds",
		task_intervals.random_stats_update
	);

	// Create an interval that ticks every `random_stats_update` seconds.
	let mut interval = interval(Duration::from_secs(task_intervals.random_stats_update));

	// Maximum number of consecutive failures before increasing wait time
	const MAX_CONSECUTIVE_FAILURES: u32 = 3;
	let mut consecutive_failures = 0;
	let base_retry_delay = Duration::from_secs(5);
	let mut update_count = 0;

	// Run the update task indefinitely.
	loop {
		// Wait for the next tick of the interval.
		trace!("Waiting for next random stats update interval tick");
		interval.tick().await;
		update_count += 1;

		let current_time = chrono::Utc::now();
		info!(
			"Starting random stats update cycle #{} at {}",
			update_count, current_time
		);

		// Update the random statistics and handle any errors.
		trace!("Calling update_random_stats function with Anilist cache");
		match update_random_stats(anilist_cache.clone()).await {
			Ok(stats) => {
				info!(
					"Random stats update cycle #{} completed successfully",
					update_count
				);
				debug!(
					"Updated random stats: anime_last_page={}, manga_last_page={}",
					stats.anime_last_page, stats.manga_last_page
				);

				// Reset failure counter on success
				if consecutive_failures > 0 {
					info!(
						"Random stats update recovered after {} consecutive failures",
						consecutive_failures
					);
					consecutive_failures = 0;
				}
			},
			Err(err) => {
				consecutive_failures += 1;
				error!(
					"Random stats update cycle #{} failed: {:#}",
					update_count, err
				);
				warn!(
					"This is consecutive failure #{} for random stats updates",
					consecutive_failures
				);

				// Log more details about the error for debugging
				debug!("Error type: {}", std::any::type_name_of_val(&err));
				debug!("Error context: {}", err.to_string());

				// Implement exponential backoff if we have consecutive failures
				if consecutive_failures > MAX_CONSECUTIVE_FAILURES {
					let delay =
						base_retry_delay.mul_f32(1.5_f32.powi(consecutive_failures as i32 - 3));
					warn!(
						"Multiple consecutive random stats update failures detected. Implementing backoff strategy."
					);
					error!("Waiting for {:?} before next attempt", delay);
					sleep(delay).await;
				}
			},
		}

		debug!(
			"Next random stats update scheduled in {} seconds",
			task_intervals.random_stats_update
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

/// Updates the random statistics by fetching the latest statistics from the Anilist API and saving them to a JSON file.
///
/// # Arguments
///
/// * `anilist_cache` - A cache for storing Anilist API responses.
///
/// # Returns
///
/// Returns the updated `RandomStat` on success, or an error on failure.

pub async fn update_random_stats(
	anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<RandomStat> {
	trace!("Starting update_random_stats function");
	debug!(
		"Attempting to load random stats from file: {}",
		RANDOM_STATS_PATH
	);

	// Try to load random stats from a JSON file.
	let mut random_stats: RandomStat = match std::fs::read_to_string(RANDOM_STATS_PATH) {
		Ok(stats) => {
			trace!(
				"Successfully read random stats file, content length: {} bytes",
				stats.len()
			);
			debug!("Parsing random stats JSON content");

			match serde_json::from_str::<RandomStat>(&stats) {
				Ok(parsed_stats) => {
					debug!(
						"Successfully parsed random stats: anime_last_page={}, manga_last_page={}",
						parsed_stats.anime_last_page, parsed_stats.manga_last_page
					);
					parsed_stats
				},
				Err(e) => {
					warn!(
						"Failed to parse random stats JSON from {}: {}",
						RANDOM_STATS_PATH, e
					);
					debug!(
						"First 100 characters of file content: {}",
						stats.chars().take(100).collect::<String>()
					);
					return Err(anyhow::anyhow!(e).context(format!(
						"Failed to parse random stats JSON from {}",
						RANDOM_STATS_PATH
					)));
				},
			}
		},
		Err(err) => {
			// Log that we're using default values and why
			if err.kind() == std::io::ErrorKind::NotFound {
				info!(
					"Random stats file not found ({}), using default values",
					RANDOM_STATS_PATH
				);
			} else {
				warn!(
					"Could not read random stats file ({}), using default values: {}",
					RANDOM_STATS_PATH, err
				);
				debug!("Error kind: {:?}", err.kind());
			}

			let default_stats = RandomStat::default();
			debug!(
				"Using default values: anime_last_page={}, manga_last_page={}",
				default_stats.anime_last_page, default_stats.manga_last_page
			);
			default_stats
		},
	};

	// Update the random statistics.
	debug!("Calling update_random function to fetch latest statistics");
	trace!(
		"Current stats before update: anime_last_page={}, manga_last_page={}",
		random_stats.anime_last_page, random_stats.manga_last_page
	);

	let start_time = chrono::Utc::now();
	random_stats = update_random(random_stats, anilist_cache)
		.await
		.with_context(|| "Failed to update random statistics from Anilist API")?;

	let elapsed = chrono::Utc::now() - start_time;
	debug!(
		"update_random completed in {} ms",
		elapsed.num_milliseconds()
	);
	debug!(
		"Updated stats: anime_last_page={}, manga_last_page={}",
		random_stats.anime_last_page, random_stats.manga_last_page
	);

	// Write the updated random statistics to a JSON file.
	trace!("Serializing random stats to JSON");
	let random_stats_json = serde_json::to_string(&random_stats)
		.with_context(|| "Failed to serialize random stats to JSON")?;
	debug!("Serialized JSON size: {} bytes", random_stats_json.len());

	trace!("Writing random stats to file: {}", RANDOM_STATS_PATH);
	match std::fs::write(RANDOM_STATS_PATH, &random_stats_json) {
		Ok(_) => debug!(
			"Successfully wrote random stats to file: {}",
			RANDOM_STATS_PATH
		),
		Err(e) => {
			warn!(
				"Failed to write random stats to file {}: {}",
				RANDOM_STATS_PATH, e
			);
			debug!("Error kind: {:?}", e.kind());
			return Err(anyhow::anyhow!(e).context(format!(
				"Failed to write random stats to file {}",
				RANDOM_STATS_PATH
			)));
		},
	}

	// Log successful update
	info!(
		"Successfully updated random stats: anime_last_page={}, manga_last_page={}",
		random_stats.anime_last_page, random_stats.manga_last_page
	);

	// Return the updated random statistics.
	trace!("Exiting update_random_stats function");
	Ok(random_stats)
}

/// Updates the random statistics by repeatedly calling `update_page` until there are no more pages to update.
///
/// # Arguments
///
/// * `random_stats` - The current random statistics.
/// * `anilist_cache` - A cache for storing Anilist API responses.
///
/// # Returns
///
/// A `Result` containing the updated random statistics or an error.

async fn update_random(
	mut random_stats: RandomStat, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<RandomStat> {
	trace!("Starting update_random function");
	debug!(
		"Initial stats: anime_last_page={}, manga_last_page={}",
		random_stats.anime_last_page, random_stats.manga_last_page
	);

	// Maximum number of consecutive failures before giving up on a category
	const MAX_FAILURES: u32 = 5;
	let mut anime_failures = 0;
	let mut manga_failures = 0;

	// Track overall performance
	let start_time = chrono::Utc::now();

	// Update anime statistics
	info!(
		"Starting anime statistics update from page {}",
		random_stats.anime_last_page
	);
	trace!("Anime update max failures threshold: {}", MAX_FAILURES);
	let mut has_more_pages = true;
	let mut pages_updated = 0;
	let anime_start_time = chrono::Utc::now();

	while has_more_pages && anime_failures < MAX_FAILURES {
		trace!(
			"Calling update_page for anime (page {})",
			random_stats.anime_last_page
		);
		let page_start_time = chrono::Utc::now();
		has_more_pages = update_page(&mut random_stats, anilist_cache.clone(), true, false).await;
		let page_elapsed = chrono::Utc::now() - page_start_time;
		trace!(
			"update_page for anime completed in {} ms",
			page_elapsed.num_milliseconds()
		);

		if has_more_pages {
			pages_updated += 1;
			debug!(
				"Successfully updated anime page {}, moving to next page",
				random_stats.anime_last_page - 1
			);

			if anime_failures > 0 {
				debug!(
					"Reset anime failure counter from {} to 0 after successful update",
					anime_failures
				);
				anime_failures = 0; // Reset failure counter on success
			}

			if pages_updated % 10 == 0 {
				info!("Anime update progress: {} pages processed", pages_updated);
			}
		} else {
			// If we didn't get more pages, it could be an error or end of data
			// The update_page function will log specific errors
			anime_failures += 1;
			warn!(
				"Anime update attempt failed for page {}, failure {}/{}",
				random_stats.anime_last_page, anime_failures, MAX_FAILURES
			);

			if anime_failures < MAX_FAILURES {
				// Wait a bit longer before retry on failure
				let retry_delay = 2 * anime_failures; // Increase delay with each failure
				debug!("Retrying anime update after {}s delay", retry_delay);
				sleep(Duration::from_secs(retry_delay.into())).await;
				has_more_pages = true; // Try again
				continue;
			} else {
				warn!(
					"Reached maximum anime update failures ({}), moving to manga updates",
					MAX_FAILURES
				);
			}
		}

		// Sleep to avoid rate limiting
		trace!("Sleeping 1s to avoid API rate limiting");
		sleep(Duration::from_secs(1)).await;
	}

	if anime_failures >= MAX_FAILURES {
		error!(
			"Exceeded maximum failures ({}) for anime statistics update",
			MAX_FAILURES
		);
		warn!("Some anime statistics may not be up to date");
	}

	let anime_elapsed = chrono::Utc::now() - anime_start_time;
	info!(
		"Completed anime statistics update, processed {} pages in {} seconds",
		pages_updated,
		anime_elapsed.num_seconds()
	);
	debug!(
		"Average time per anime page: {} ms",
		if pages_updated > 0 {
			anime_elapsed.num_milliseconds() / pages_updated as i64
		} else {
			0
		}
	);

	// Update manga statistics
	info!(
		"Starting manga statistics update from page {}",
		random_stats.manga_last_page
	);
	trace!("Manga update max failures threshold: {}", MAX_FAILURES);
	has_more_pages = true;
	pages_updated = 0;
	let manga_start_time = chrono::Utc::now();

	while has_more_pages && manga_failures < MAX_FAILURES {
		trace!(
			"Calling update_page for manga (page {})",
			random_stats.manga_last_page
		);
		let page_start_time = chrono::Utc::now();
		has_more_pages = update_page(&mut random_stats, anilist_cache.clone(), false, true).await;
		let page_elapsed = chrono::Utc::now() - page_start_time;
		trace!(
			"update_page for manga completed in {} ms",
			page_elapsed.num_milliseconds()
		);

		if has_more_pages {
			pages_updated += 1;
			debug!(
				"Successfully updated manga page {}, moving to next page",
				random_stats.manga_last_page - 1
			);

			if manga_failures > 0 {
				debug!(
					"Reset manga failure counter from {} to 0 after successful update",
					manga_failures
				);
				manga_failures = 0; // Reset failure counter on success
			}

			if pages_updated % 10 == 0 {
				info!("Manga update progress: {} pages processed", pages_updated);
			}
		} else {
			// If we didn't get more pages, it could be an error or end of data
			manga_failures += 1;
			warn!(
				"Manga update attempt failed for page {}, failure {}/{}",
				random_stats.manga_last_page, manga_failures, MAX_FAILURES
			);

			if manga_failures < MAX_FAILURES {
				// Wait a bit longer before retry on failure
				let retry_delay = 2 * manga_failures; // Increase delay with each failure
				debug!("Retrying manga update after {}s delay", retry_delay);
				sleep(Duration::from_secs(retry_delay.into())).await;
				has_more_pages = true; // Try again
				continue;
			} else {
				warn!("Reached maximum manga update failures ({})", MAX_FAILURES);
			}
		}

		// Sleep to avoid rate limiting
		trace!("Sleeping 1s to avoid API rate limiting");
		sleep(Duration::from_secs(1)).await;
	}

	if manga_failures >= MAX_FAILURES {
		error!(
			"Exceeded maximum failures ({}) for manga statistics update",
			MAX_FAILURES
		);
		warn!("Some manga statistics may not be up to date");
	}

	let manga_elapsed = chrono::Utc::now() - manga_start_time;
	info!(
		"Completed manga statistics update, processed {} pages in {} seconds",
		pages_updated,
		manga_elapsed.num_seconds()
	);
	debug!(
		"Average time per manga page: {} ms",
		if pages_updated > 0 {
			manga_elapsed.num_milliseconds() / pages_updated as i64
		} else {
			0
		}
	);

	let total_elapsed = chrono::Utc::now() - start_time;
	info!(
		"Total statistics update completed in {} seconds",
		total_elapsed.num_seconds()
	);
	debug!(
		"Final stats: anime_last_page={}, manga_last_page={}",
		random_stats.anime_last_page, random_stats.manga_last_page
	);
	trace!("Exiting update_random function");

	Ok(random_stats)
}

/// Updates a page of random statistics by fetching data from the Anilist API.
///
/// # Arguments
///
/// * `random_stats` - The current random statistics to update.
/// * `anilist_cache` - A cache for storing Anilist API responses.
/// * `update_anime` - Whether to update anime statistics.
/// * `update_manga` - Whether to update manga statistics.
///
/// # Returns
///
/// Returns true if there are more pages to update, false otherwise.
async fn update_page(
	random_stats: &mut RandomStat, anilist_cache: Arc<RwLock<Cache<String, String>>>,
	update_anime: bool, update_manga: bool,
) -> bool {
	// If neither anime nor manga updates are requested, return early
	if !update_anime && !update_manga {
		trace!("Neither anime nor manga updates requested, returning early");
		return false;
	}

	let page_type = if update_anime { "anime" } else { "manga" };
	let page_num = if update_anime {
		random_stats.anime_last_page
	} else {
		random_stats.manga_last_page
	};

	trace!("Starting update_page for {} page {}", page_type, page_num);
	debug!(
		"Preparing GraphQL request for {} statistics page {}",
		page_type, page_num
	);

	// Track API request performance
	let request_start_time = chrono::Utc::now();

	let data = if update_anime {
		let var = AnimeStatVariables {
			page: Some(random_stats.anime_last_page),
		};

		trace!(
			"Building AnimeStat GraphQL operation with variables: {:?}",
			var.page
		);
		let operation = AnimeStat::build(var);

		trace!("Making Anilist API request for anime statistics");
		let data: Result<GraphQlResponse<AnimeStat>> =
			make_request_anilist(operation, false, anilist_cache.clone())
				.await
				.with_context(|| {
					format!(
						"Failed to fetch {} statistics for page {}",
						page_type, page_num
					)
				});

		data
	} else {
		let var = MangaStatVariables {
			page: Some(random_stats.manga_last_page),
		};

		trace!(
			"Building MangaStat GraphQL operation with variables: {:?}",
			var.page
		);
		let operation = MangaStat::build(var);

		trace!("Making Anilist API request for manga statistics");
		let data: Result<GraphQlResponse<AnimeStat>> =
			make_request_anilist(operation, false, anilist_cache.clone())
				.await
				.with_context(|| {
					format!(
						"Failed to fetch {} statistics for page {}",
						page_type, page_num
					)
				});

		data
	};

	let request_elapsed = chrono::Utc::now() - request_start_time;
	debug!(
		"Anilist API request for {} page {} completed in {} ms",
		page_type,
		page_num,
		request_elapsed.num_milliseconds()
	);

	let data = match data {
		Ok(data) => {
			trace!(
				"Successfully received API response for {} page {}",
				page_type, page_num
			);
			data
		},
		Err(err) => {
			// Log the error with details about which page failed
			error!(
				"Error updating {} stats for page {}: {:#}",
				page_type, page_num, err
			);
			debug!("Error type: {}", std::any::type_name_of_val(&err));

			// Check for specific error patterns
			let err_string = err.to_string();
			if err_string.contains("timeout") {
				warn!(
					"API timeout detected for {} page {}. The external service might be experiencing high load.",
					page_type, page_num
				);
			} else if err_string.contains("rate") {
				warn!(
					"Possible rate limiting detected for {} page {}. Consider increasing the update interval.",
					page_type, page_num
				);
			} else if err_string.contains("network") || err_string.contains("connection") {
				warn!(
					"Network connectivity issue detected for {} page {}. Check internet connection.",
					page_type, page_num
				);
			}

			// Return false to indicate no more pages (will retry on next scheduled run)
			trace!("Returning false due to API request error");
			return false;
		},
	};

	trace!("Parsing API response to extract pagination information");

	// Extract has_next_page value with better error handling
	let has_next_page = match &data.data {
		Some(data) => {
			trace!("Found data in API response");
			match &data.site_statistics {
				Some(site_statistics) => {
					trace!("Found site_statistics in API response");
					match &site_statistics.manga {
						Some(manga) => {
							trace!("Found manga data in API response");
							match &manga.page_info {
								Some(page_info) => {
									trace!("Found page_info in API response");
									let result = page_info.has_next_page.unwrap_or(false);
									if result {
										debug!(
											"Found more pages to process for {} after page {}",
											page_type, page_num
										);
									} else {
										debug!(
											"No more pages available for {} after page {}",
											page_type, page_num
										);
									}
									result
								},
								None => {
									error!(
										"Missing page_info in API response for {} page {}",
										page_type, page_num
									);
									warn!("API response structure may have changed, check schema");
									trace!("Response data structure: {:?}", data);
									false
								},
							}
						},
						None => {
							error!(
								"Missing manga data in API response for {} page {}",
								page_type, page_num
							);
							warn!("API response structure may have changed, check schema");
							trace!("Response data structure: {:?}", data);
							false
						},
					}
				},
				None => {
					error!(
						"Missing site_statistics in API response for {} page {}",
						page_type, page_num
					);
					warn!("API response structure may have changed, check schema");
					trace!("Response data structure: {:?}", data);
					false
				},
			}
		},
		None => {
			error!(
				"Empty data in API response for {} page {}",
				page_type, page_num
			);
			warn!("API may have returned an empty response, check for service issues");
			trace!("Full response: {:?}", data);
			false
		},
	};

	if has_next_page {
		if update_anime {
			random_stats.anime_last_page += 1;
			debug!(
				"Incrementing anime_last_page to {}",
				random_stats.anime_last_page
			);
		} else {
			random_stats.manga_last_page += 1;
			debug!(
				"Incrementing manga_last_page to {}",
				random_stats.manga_last_page
			);
		}
		trace!("Returning true to indicate more pages available");
	} else {
		info!(
			"Reached last page for {} stats: page {}",
			page_type, page_num
		);
		trace!("Returning false to indicate no more pages available");
	}

	has_next_page
}
