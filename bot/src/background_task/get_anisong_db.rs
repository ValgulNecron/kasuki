// Import necessary libraries and modules
use crate::database::prelude::AnimeSong;
use anyhow::{Context, Result};
use futures::future::join_all;
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, trace, warn};

/// Fetches anime song data from anisongdb.com and stores it in the database.
///
/// This function makes concurrent requests to the anisongdb API to retrieve
/// anime song information and stores valid entries in the database. It uses
/// a semaphore to limit the number of concurrent requests.
///
/// # Arguments
///
/// * `connection` - An Arc-wrapped database connection
///
/// # Returns
///
/// * `Ok(())` - If the operation completes successfully
/// * `Err(Error)` - If there is an error during the operation
///
/// # Errors
///
/// This function can return errors if:
/// - There are issues with acquiring semaphore permits
/// - API requests fail
/// - Response parsing fails
/// - Database operations fail
pub async fn get_anisong(connection: Arc<DatabaseConnection>) -> Result<usize> {
	info!("Starting anisong database update task");

	// Create a reusable HTTP client for all requests
	// This is more efficient than creating a new client for each request
	// as it can reuse connections and maintain a connection pool
	let client = Arc::new(Client::new());

	// Create a semaphore to limit concurrent API requests
	// This prevents overwhelming the API server and potential rate limiting
	// The value 10 was chosen as a balance between throughput and server load
	let semaphore = Arc::new(Semaphore::new(10)); 

	// Collection to store all the spawned task futures
	let mut futures = Vec::new();

	// Start with ANN ID 1 and increment
	let mut i = 1;

	// Set an upper bound for ANN IDs to prevent infinite processing
	// 100,000 is a reasonable limit based on the current ANN database size
	// This was reduced from 100,000,000 in previous versions for efficiency
	let max_count: i64 = 100_000; 

	debug!("Will process anime songs with IDs from 1 to {}", max_count);

	// Process each ANN ID in sequence, but handle the API requests concurrently
	while i <= max_count {
		// Acquire a permit from the semaphore before spawning a new task
		// This ensures we never have more than 10 concurrent requests
		// The acquire_owned() method is used because the permit needs to be moved into the task
		let permit = semaphore.clone().acquire_owned().await
			.context(format!("Failed to acquire semaphore permit for ANN ID {}", i))?;

		// Clone the shared resources for use in the spawned task
		let client = client.clone();
		let connection = connection.clone();

		let ann_id = i;

		// Spawn a new task to process this ANN ID concurrently
		// This allows us to make progress on other IDs while waiting for API responses
		let future = tokio::spawn(async move {
			// Keep the permit alive for the duration of the task
			// When this variable is dropped at the end of the task, the permit is released
			// allowing another task to acquire it
			let _permit = permit; 

			// Call the helper function to process this specific ANN ID
			match process_anisong(client, connection, ann_id).await {
				Ok(raw_anisong) => raw_anisong,
				Err(e) => {
					error!("Error processing anime song for ANN ID {}: {}", ann_id, e);
					// Return an empty vector on error to maintain consistent return type
					Vec::new()
				}
			}
		});

		/// Helper function to process a single anime song request with proper error handling
		///
		/// This nested function encapsulates the logic for:
		/// 1. Making the API request to anisongdb.com for a specific ANN ID
		/// 2. Parsing the response and extracting anime song data
		/// 3. Filtering out entries without Anilist IDs
		/// 4. Inserting or updating records in the database
		///
		/// By defining this function inside the main function, we can access the
		/// outer function's context while keeping the code modular and focused.
		async fn process_anisong(
			client: Arc<Client>, 
			connection: Arc<DatabaseConnection>, 
			ann_id: i64
		) -> Result<Vec<RawAniSongDB>> {
			debug!("Requesting anime song data for ANN ID: {}", ann_id);

			// Make the API request to anisongdb.com
			// We use POST with JSON payload to request songs for a specific ANN ID
			// The ignore_duplicate parameter is set to false to get all songs
			let response = client
				.post("https://anisongdb.com/api/annId_request")
				.header("Content-Type", "application/json")
				.header("Accept", "application/json")
				.json(&json!({
					"annId": ann_id,
					"ignore_duplicate": false,
				}))
				.send()
				.await
				.context(format!("Failed to request anime song data for ANN ID {}", ann_id))?;

			trace!("Received response for ANN ID: {}, status: {}", ann_id, response.status());
			trace!(?response);

			// Extract the response body as text
			// This is a separate step from JSON parsing to help with debugging
			// If JSON parsing fails, we can still log the raw text
			let json = response.text().await
				.context(format!("Failed to parse response text for ANN ID {}", ann_id))?;

			trace!("Successfully parsed response text for ANN ID: {}", ann_id);

			// Parse the JSON text into our RawAniSongDB struct
			// The API returns an array of songs, even for a single ANN ID
			// because one anime can have multiple songs (OP, ED, etc.)
			let raw_anisong: Vec<RawAniSongDB> = serde_json::from_str(&json)
				.context(format!("Failed to parse JSON for ANN ID {}", ann_id))?;

			debug!("Successfully parsed {} anime songs for ANN ID: {}", raw_anisong.len(), ann_id);

			// Track statistics for logging and reporting
			let mut processed_count = 0;
			let mut success_count = 0;

			// Process each anime song in the response
			for anisong in raw_anisong.clone() {
				processed_count += 1;

				// Skip songs without an Anilist ID since we need it for linking
				// to other parts of the application that use Anilist as reference
				if anisong.linked_ids.anilist.is_none() {
					trace!("Skipping anime song with ANN ID {} due to missing Anilist ID", anisong.ann_id);
					continue;
				}

				// Unwrap is safe here because we checked for None above
				let anilist_id = anisong.linked_ids.anilist.unwrap();
				debug!("Processing anime song: '{}' (ANN ID: {}, Anilist ID: {})", 
					anisong.song_name, anisong.ann_id, anilist_id);

				// Prepare the database model from the API response
				// We transform the raw API data into our database schema format
				// Note: We prepend URLs with the base domain and handle optional fields
				let anime_song_model = crate::database::anime_song::ActiveModel {
					// Primary keys and identifiers
					anilist_id: Set(anilist_id.to_string()),
					ann_id: Set(anisong.ann_id.to_string()),
					ann_song_id: Set(anisong.ann_song_id.to_string()),

					// Anime metadata
					anime_en_name: Set(anisong.anime_en_name.clone()),
					anime_jp_name: Set(anisong.anime_jp_name.clone()),
					// Join alternative names with commas if present, otherwise empty string
					anime_alt_name: Set(anisong.anime_alt_name.unwrap_or_default().join(", ")),

					// Song metadata
					song_type: Set(anisong.song_type.clone()),
					song_name: Set(anisong.song_name.clone()),

					// Media URLs - prepend the base domain to the file paths
					// Use empty string as fallback if the URL is not provided
					hq: Set(format!(
						"https://files.catbox.moe/{}",
						anisong.hq.unwrap_or_default()
					)),
					mq: Set(format!(
						"https://files.catbox.moe/{}",
						anisong.mq.unwrap_or_default()
					)),
					audio: Set(format!(
						"https://files.catbox.moe/{}",
						anisong.audio.unwrap_or_default()
					)),
				};

				// Insert the record with conflict resolution strategy
				// This implements an "upsert" pattern (insert or update if exists)
				let result = AnimeSong::insert(anime_song_model)
					// Define conflict resolution based on the composite key
					// If a record with the same AnilistId, AnnId, and AnnSongId exists:
					.on_conflict(
						sea_orm::sea_query::OnConflict::columns([
							// These three columns form our composite unique key
							crate::database::anime_song::Column::AnilistId,
							crate::database::anime_song::Column::AnnId,
							crate::database::anime_song::Column::AnnSongId,
						])
						// Update these columns with the new values
						// This ensures we keep the latest data while preserving the record
						.update_columns([
							crate::database::anime_song::Column::AnimeEnName,
							crate::database::anime_song::Column::AnimeJpName,
							crate::database::anime_song::Column::AnimeAltName,
							crate::database::anime_song::Column::SongType,
							crate::database::anime_song::Column::SongName,
							crate::database::anime_song::Column::Hq,
							crate::database::anime_song::Column::Mq,
							crate::database::anime_song::Column::Audio,
						])
						.to_owned(),
					)
					// Execute the query on the database connection
					.exec(&*connection)
					.await
					.context(format!("Failed to insert anime song '{}' for ANN ID {}", 
						anisong.song_name, anisong.ann_id))?;

				success_count += 1;
				trace!("Successfully inserted/updated anime song: '{}' for anime: '{}'", 
					anisong.song_name, anisong.anime_en_name);
			}

			if processed_count > 0 {
				debug!("Processed {} anime songs for ANN ID {}, successfully inserted/updated: {}", 
					processed_count, ann_id, success_count);
			}

			// Return the raw anime song data for potential further processing
			// and to track the total number of processed items
			Ok(raw_anisong)
		}

 	// Add this task to our collection of futures
 	futures.push(future);

 	// Move to the next ANN ID
 	i += 1;
 }

 // Wait for all futures to complete and collect results
 // join_all runs all futures to completion and collects their results
 // This is more efficient than awaiting each future individually
 info!("Waiting for all {} anime song processing tasks to complete", futures.len());
 let results = join_all(futures).await;

 // Count total processed and successful operations
 // This aggregates the results from all tasks:
 // 1. Filter out tasks that failed (using filter_map and as_ref().ok())
 // 2. Flatten the Vec<Vec<RawAniSongDB>> into a single iterator
 // 3. Count the total number of items
 let total_processed = results.iter().filter_map(|r| r.as_ref().ok()).flatten().count();

 // Log the final results
 info!("Anisong database update task completed. Processed {} anime song entries.", total_processed);
 debug!("Task success rate: {}/{} ({:.1}%)", 
 	results.iter().filter(|r| r.is_ok()).count(),
 	results.len(),
 	(results.iter().filter(|r| r.is_ok()).count() as f64 / results.len() as f64) * 100.0);

 // Return the total number of processed items
 // This allows the caller to track the progress over time
 Ok(total_processed)
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawAniSongDB {
	#[serde(rename = "annId")]
	ann_id: i64,
	#[serde(rename = "annSongId")]
	ann_song_id: i64,
	#[serde(rename = "animeENName")]
	anime_en_name: String,
	#[serde(rename = "animeJPName")]
	anime_jp_name: String,
	#[serde(rename = "animeAltName")]
	anime_alt_name: Option<Vec<String>>,
	linked_ids: Linked,
	#[serde(rename = "songType")]
	song_type: String,
	#[serde(rename = "songName")]
	song_name: String,
	#[serde(rename = "HQ")]
	hq: Option<String>,
	#[serde(rename = "MQ")]
	mq: Option<String>,
	audio: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Linked {
	anilist: Option<i64>,
}
