use std::collections::HashMap;
use std::sync::Arc;

// Import necessary libraries and modules
use anyhow::{Context as AnyhowContext, Result};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, trace};

// App is a struct that represents a Steam app
#[derive(Debug, Deserialize, Clone)]
pub struct App {
	// app_id is the id of the app
	#[serde(rename = "appid")]
	pub app_id: u128,
	// name is the name of the app
	pub name: String,
}

/// `get_game` is an asynchronous function that gets the list of Steam apps and stores them in the APPS constant.
/// It returns the number of new entries added to the cache.
///
/// # Arguments
///
/// * `apps_data` - An Arc-wrapped RwLock containing a HashMap of game names to app IDs
///
/// # Returns
///
/// * `Ok(usize)` - The number of new entries added to the cache
/// * `Err(Error)` - If there is an error during the operation
///
/// # Errors
///
/// This function will return an error if it encounters any issues while:
/// - Making the HTTP request to the Steam API
/// - Parsing the response body
/// - Deserializing the JSON
/// - Updating the apps cache

pub async fn get_game(apps_data: Arc<RwLock<HashMap<String, u128>>>) -> Result<usize> {
	// Log the start of the process
	debug!("Started Steam game data update process");
	trace!("Preparing to fetch game list from Steam API");

	// Define the URL for the Steam API
	let url = "https://api.steampowered.com/ISteamApps/GetAppList/v0002/?format=json";
	debug!("Using Steam API URL: {}", url);

	// Make the HTTP request with proper error context
	let response = reqwest::get(url).await
		.context("Failed to connect to Steam API")?;

	// Get the response status for logging
	debug!("Received response from Steam API with status: {}", response.status());

	// Get the response body with proper error context
	let body = response.text().await
		.context("Failed to read Steam API response body")?;

	trace!("Successfully retrieved response body ({} bytes)", body.len());

	// Parse the response body as JSON with proper error context
	let json: Value = serde_json::from_str(&body)
		.context("Failed to parse Steam API response as JSON")?;

	debug!("Successfully parsed Steam API response as JSON");

	// Ensure the expected JSON structure exists
	if !json.get("applist").and_then(|v| v.get("apps")).is_some() {
		return Err(anyhow::anyhow!("Steam API response missing expected 'applist.apps' structure"))
			.context("Invalid Steam API response format")?;
	}

	// Deserialize the JSON into a vector of App structs with proper error context
	let apps: Vec<App> = serde_json::from_value(json["applist"]["apps"].clone())
		.context("Failed to deserialize Steam app list from JSON")?;

	debug!("Successfully deserialized {} Steam apps from JSON", apps.len());

	// Get the current size of the apps cache for comparison
	let current_size = {
		let read_guard = apps_data.read().await;
		read_guard.len()
	};

	// Clear the apps cache and insert the new apps
	let mut write_guard = apps_data.write().await;
	trace!("Acquired write lock on apps cache");

	// Clear the existing data and free memory
	write_guard.clear();
	write_guard.shrink_to_fit();
	trace!("Cleared existing apps cache");

	// Convert the vector of apps into a hashmap
	let app_map: HashMap<String, u128> = apps
		.iter()
		.map(|app| (app.name.clone(), app.app_id))
		.collect();

	// Update the cache with the new data
	*write_guard = app_map;

	// Optimize memory usage
	write_guard.shrink_to_fit();

	// Calculate how many new entries were added
	let new_size = write_guard.len();
	let new_entries = new_size;

	// Release the write lock
	drop(write_guard);
	trace!("Released write lock on apps cache");

	debug!("Successfully updated Steam game cache: {} entries", new_size);
	debug!("Steam game data update process completed");

	// Return the number of new entries
	Ok(new_entries)
}
