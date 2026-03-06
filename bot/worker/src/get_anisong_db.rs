use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use governor::{Quota, RateLimiter};
use reqwest::{Client, StatusCode};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use serde_json::json;
use shared::database::prelude::AnimeSong;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, trace, warn};

const DEFAULT_MAX_ANN_ID: i64 = 100_000;
const CONCURRENCY: usize = 10;
const REQUESTS_PER_SECOND: u32 = 20;
const BURST_SIZE: u32 = 5;
const MAX_RETRIES: u32 = 5;

pub async fn get_anisong(connection: Arc<DatabaseConnection>) -> Result<usize> {
	let start_time = Instant::now();
	info!("Starting anisong database update");

	let client = Arc::new(Client::new());
	let limiter = Arc::new(RateLimiter::direct(
		Quota::per_second(NonZeroU32::new(REQUESTS_PER_SECOND).unwrap())
			.allow_burst(NonZeroU32::new(BURST_SIZE).unwrap()),
	));

	let total_processed = Arc::new(AtomicUsize::new(0));
	let total_errors = Arc::new(AtomicUsize::new(0));

	stream::iter(1..=DEFAULT_MAX_ANN_ID)
		.map(|ann_id| {
			let client = client.clone();
			let connection = connection.clone();
			let limiter = limiter.clone();
			let total_processed = total_processed.clone();
			let total_errors = total_errors.clone();

			async move {
				match process_ann_id(&client, &connection, ann_id, &limiter).await {
					Ok(count) => {
						total_processed.fetch_add(count, Ordering::Relaxed);
					},
					Err(e) => {
						total_errors.fetch_add(1, Ordering::Relaxed);
						debug!("Error processing ANN ID {}: {:#}", ann_id, e);
					},
				}
			}
		})
		.buffer_unordered(CONCURRENCY)
		.collect::<()>()
		.await;

	let processed = total_processed.load(Ordering::Relaxed);
	let errors = total_errors.load(Ordering::Relaxed);
	let elapsed = start_time.elapsed();

	info!(
		"Anisong update done: {} songs processed, {} errors, {:.1}s elapsed",
		processed,
		errors,
		elapsed.as_secs_f64()
	);

	Ok(processed)
}

async fn process_ann_id(
	client: &Client, connection: &DatabaseConnection, ann_id: i64,
	limiter: &RateLimiter<
		governor::state::NotKeyed,
		governor::state::InMemoryState,
		governor::clock::DefaultClock,
	>,
) -> Result<usize> {
	let songs = fetch_ann_id(client, ann_id, limiter).await?;

	if songs.is_empty() {
		return Ok(0);
	}

	let mut count = 0;
	for song in songs {
		let anilist_id = match song.linked_ids.anilist {
			Some(id) => id,
			None => continue,
		};

		let model = shared::database::anime_song::ActiveModel {
			anilist_id: Set(anilist_id.to_string()),
			ann_id: Set(song.ann_id.to_string()),
			ann_song_id: Set(song.ann_song_id.to_string()),
			anime_en_name: Set(song.anime_en_name),
			anime_jp_name: Set(song.anime_jp_name),
			anime_alt_name: Set(song.anime_alt_name.unwrap_or_default().join(", ")),
			song_type: Set(song.song_type),
			song_name: Set(song.song_name),
			hq: Set(catbox_url(song.hq)),
			mq: Set(catbox_url(song.mq)),
			audio: Set(catbox_url(song.audio)),
		};

		AnimeSong::insert(model)
			.on_conflict(
				sea_orm::sea_query::OnConflict::columns([
					shared::database::anime_song::Column::AnilistId,
					shared::database::anime_song::Column::AnnId,
					shared::database::anime_song::Column::AnnSongId,
				])
				.update_columns([
					shared::database::anime_song::Column::AnimeEnName,
					shared::database::anime_song::Column::AnimeJpName,
					shared::database::anime_song::Column::AnimeAltName,
					shared::database::anime_song::Column::SongType,
					shared::database::anime_song::Column::SongName,
					shared::database::anime_song::Column::Hq,
					shared::database::anime_song::Column::Mq,
					shared::database::anime_song::Column::Audio,
				])
				.to_owned(),
			)
			.exec(connection)
			.await
			.with_context(|| format!("DB upsert failed for ANN ID {}", ann_id))?;

		count += 1;
	}

	if count > 0 {
		trace!("Upserted {} songs for ANN ID {}", count, ann_id);
	}

	Ok(count)
}

async fn fetch_ann_id(
	client: &Client, ann_id: i64,
	limiter: &RateLimiter<
		governor::state::NotKeyed,
		governor::state::InMemoryState,
		governor::clock::DefaultClock,
	>,
) -> Result<Vec<RawAniSongDB>> {
	let mut retries = 0;

	loop {
		limiter.until_ready().await;

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
			.with_context(|| format!("Request failed for ANN ID {}", ann_id))?;

		if response.status() == StatusCode::TOO_MANY_REQUESTS {
			retries += 1;
			if retries > MAX_RETRIES {
				anyhow::bail!("Rate limited too many times for ANN ID {}", ann_id);
			}

			let delay = response
				.headers()
				.get("retry-after")
				.and_then(|h| h.to_str().ok())
				.and_then(|s| s.parse::<u64>().ok())
				.unwrap_or_else(|| 2u64.pow(retries));

			warn!(
				"Rate limited on ANN ID {}, retry {}/{}, waiting {}s",
				ann_id, retries, MAX_RETRIES, delay
			);
			tokio::time::sleep(Duration::from_secs(delay)).await;
			continue;
		}

		let text = response
			.text()
			.await
			.with_context(|| format!("Failed to read response for ANN ID {}", ann_id))?;

		let songs: Vec<RawAniSongDB> = serde_json::from_str(&text)
			.with_context(|| format!("Failed to parse JSON for ANN ID {}", ann_id))?;

		return Ok(songs);
	}
}

fn catbox_url(filename: Option<String>) -> String {
	match filename {
		Some(name) if !name.is_empty() => format!("https://files.catbox.moe/{}", name),
		_ => String::new(),
	}
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
