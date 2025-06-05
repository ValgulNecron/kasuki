// Import necessary libraries and modules
use crate::database::prelude::AnimeSong;
use futures::future::join_all;
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::error;
use tracing::trace;

pub async fn get_anisong(connection: Arc<DatabaseConnection>) {
	let client = Arc::new(Client::new());
	let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent tasks
	let mut futures = Vec::new();
	let mut i = 1;
	let max_count: i64 = 100_000; // Reduced from 100_000_000 to a more reasonable value

	while i <= max_count {
		let permit = semaphore.clone().acquire_owned().await.unwrap();
		let client = client.clone();
		let connection = connection.clone();

		let future = tokio::spawn(async move {
			let _permit = permit; // Keep permit alive for the duration of the task

			let response = match client
				.post("https://anisongdb.com/api/annId_request")
				.header("Content-Type", "application/json")
				.header("Accept", "application/json")
				.json(&json!({
					"annId": i,
					"ignore_duplicate": false,
				}))
				.send()
				.await
			{
				Ok(res) => res,
				Err(e) => {
					error!("Failed to get anisong db {}......... /{}", e, i);
					return Vec::new();
				},
			};
			trace!(?i);
			trace!(?response);
			let json = match response.text().await {
				Ok(res) => res,
				Err(e) => {
					error!("Failed to get anisong db {}", e);
					return Vec::new();
				},
			};

			let raw_anisong: Vec<RawAniSongDB> = match serde_json::from_str(&json) {
				Ok(res) => res,
				Err(e) => {
					error!("Failed to get anisong db {}", e);
					return Vec::new();
				},
			};

			for anisong in raw_anisong.clone() {
				if anisong.linked_ids.anilist.is_none() {
					continue;
				}

				match AnimeSong::insert(crate::database::anime_song::ActiveModel {
					anilist_id: Set(anisong.linked_ids.anilist.unwrap().to_string()),
					ann_id: Set(anisong.ann_id.to_string()),
					ann_song_id: Set(anisong.ann_song_id.to_string()),
					anime_en_name: Set(anisong.anime_en_name),
					anime_jp_name: Set(anisong.anime_jp_name),
					anime_alt_name: Set(anisong.anime_alt_name.unwrap_or_default().join(", ")),
					song_type: Set(anisong.song_type),
					song_name: Set(anisong.song_name),
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
				})
				.on_conflict(
					sea_orm::sea_query::OnConflict::columns([
						crate::database::anime_song::Column::AnilistId,
						crate::database::anime_song::Column::AnnId,
						crate::database::anime_song::Column::AnnSongId,
					])
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
				.exec(&*connection)
				.await
				{
					Ok(_) => {},
					Err(e) => error!("Failed to insert anisong. {}", e),
				};
			}

			raw_anisong
		});

		futures.push(future);
		i += 1;
	}
	join_all(futures).await;
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
