use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use std::collections::HashMap;
use std::sync::Arc;
// Import necessary libraries and modules
use crate::config::DbConfig;
use crate::database::prelude::{AnimeSong, GuildSubscription};
use crate::get_url;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::error;
use tracing::trace;

#[derive(Debug)]
pub struct AniSongDB {
	ann_id: i64,
	ann_song_id: i64,
	anime_en_name: String,
	anime_jp_name: String,
	anime_alt_name: Option<Vec<String>>,
	song_type: String,
	song_name: String,
	hq: Option<String>,
	mq: Option<String>,
	audio: Option<String>,
}

pub async fn get_anisong(
	anisong_list: Arc<RwLock<HashMap<String, AniSongDB>>>, db: DbConfig,
) -> Arc<RwLock<HashMap<String, AniSongDB>>> {
	let connection = match sea_orm::Database::connect(get_url(db)).await {
		Ok(connection) => connection,
		Err(e) => {
			error!("Failed to connect to the database. {}", e);

			return anisong_list;
		},
	};
	let client = Client::new();
	let mut i = 1;
	let mut raw_anisongs = Vec::new();
	let mut count = 0;
	let max_count = 100_000_000;
	loop {
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
			Ok(res) => {
				if count >= max_count {
					break;
				} else {
					count = count + 1;
				}
				res
			},
			Err(e) => {
				error!("Failed to get anisong db {}......... /{}", e, i);
				if count >= max_count {
					break;
				} else {
					count = count + 1;
					continue;
				}
			},
		};
		trace!(?response);
		let json = match response.text().await {
			Ok(res) => {
				if count >= max_count {
					break;
				} else {
					count = count + 1;
				}
				res
			},
			Err(e) => {
				error!("Failed to get anisong db {}", e);
				if count >= max_count {
					break;
				} else {
					count = count + 1;
					continue;
				}
			},
		};
		trace!(?json);
		let mut raw_anisong: Vec<RawAniSongDB> = match serde_json::from_str(&json) {
			Ok(res) => {
				if count >= max_count {
					break;
				} else {
					count = count + 1;
				}
				res
			},
			Err(e) => {
				error!("Failed to get anisong db {}", e);
				if count >= max_count {
					break;
				} else {
					count = count + 1;
					continue;
				}
			},
		};

		for anisong in raw_anisong.clone() {
			match AnimeSong::insert(crate::database::anime_song::ActiveModel {
				anilist_id: Set(anisong.linked_ids.anilist.to_string()),
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
			.exec(&connection)
			.await
			{
				Ok(_) => {},
				Err(e) => error!("Failed to insert anisong. {}", e),
			};
		}

		raw_anisongs.append(&mut raw_anisong);
		i = i + 1;
	}

	let new_anisong: HashMap<String, AniSongDB> = raw_anisongs
		.into_iter()
		.map(|ani_song| {
			let data: AniSongDB = AniSongDB {
				ann_id: ani_song.ann_id,
				ann_song_id: ani_song.ann_song_id,
				anime_en_name: ani_song.anime_en_name,
				anime_jp_name: ani_song.anime_jp_name,
				anime_alt_name: ani_song.anime_alt_name,
				song_type: ani_song.song_type,
				song_name: ani_song.song_name,
				hq: ani_song.hq,
				mq: ani_song.mq,
				audio: ani_song.audio,
			};

			(ani_song.linked_ids.anilist.to_string(), data)
		})
		.collect();

	trace!("{:?}", new_anisong);

	let mut anisong_list_guard = anisong_list.write().await;
	*anisong_list_guard = new_anisong;
	drop(anisong_list_guard);
	anisong_list
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
	anilist: i64,
}
