use shared::config::TaskIntervalConfig;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

/// Maintains a list of blacklisted users by periodically fetching from a remote source.
#[tracing::instrument(skip(blacklist_lock, task_intervals), level = "info")]
pub async fn update_user_blacklist(
	blacklist_lock: Arc<RwLock<HashSet<String>>>, task_intervals: TaskIntervalConfig,
) {
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

	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(10))
		.build()
		.unwrap_or_else(|e| {
			warn!("Failed to create custom HTTP client with timeout: {}", e);
			warn!("Using default reqwest client instead");
			reqwest::Client::new()
		});
	debug!("HTTP client initialized for blacklist updates");

	loop {
		trace!("Waiting for next blacklist update interval tick");
		interval.tick().await;
		update_count += 1;

		let current_time = chrono::Utc::now();
		info!(
			"Starting blacklist update cycle #{} at {}",
			update_count, current_time
		);

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

		info!("Fetching blacklist from remote source");
		trace!("HTTP GET request to: {}", blacklist_url);

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

				debug!("Error type: {}", std::any::type_name_of_val(&e));

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

		trace!("Parsing JSON response from blacklist request");
		let blacklist_json: serde_json::Value = match blacklist_response.json().await {
			Ok(json) => {
				debug!("Successfully parsed blacklist JSON");
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

		trace!("Extracting user_id array from JSON response");
		let user_ids: HashSet<String> = match blacklist_json["user_id"].as_array() {
			Some(arr) => {
				debug!(
					"Found user_id array in blacklist with {} entries",
					arr.len()
				);
				trace!(
					"First 5 user IDs in array: {:?}",
					arr.iter()
						.take(5)
						.map(|v| v.to_string())
						.collect::<Vec<String>>()
				);
				arr
			},
			None => {
				consecutive_failures += 1;
				error!("Failed to get user_id array from blacklist JSON");
				warn!(
					"This is consecutive failure #{} for blacklist updates",
					consecutive_failures
				);
				debug!(
					"JSON keys available: {:?}",
					blacklist_json
						.as_object()
						.map(|o| o.keys().cloned().collect::<Vec<String>>())
				);
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
		}
		.iter()
		.map(|id| match id.as_str() {
			Some(id) => id.to_string(),
			None => {
				error!("Found non-string value in user_id array");
				debug!("Invalid value type: {}", id.to_string());
				"".to_string()
			},
		})
		.filter(|id| !id.is_empty())
		.collect();

		info!("Updating blacklist with {} users", user_ids.len());
		trace!("Acquiring write lock on blacklist");

		let mut blacklist = blacklist_lock.write().await;
		trace!("Write lock acquired");

		trace!(
			"Clearing existing blacklist with {} entries",
			blacklist.len()
		);
		blacklist.clear();

		trace!("Shrinking blacklist capacity");
		blacklist.shrink_to_fit();

		trace!("Updating blacklist with new user IDs");
		*blacklist = user_ids;

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

		trace!("Write lock released");
	}
}
