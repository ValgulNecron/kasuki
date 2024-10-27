use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct CompareLocalised {
	pub affinity: String,
	pub more_anime: String,
	pub same_anime: String,
	pub more_watch_time: String,
	pub same_watch_time: String,
	pub genre_anime: String,
	pub same_genre_anime: String,
	pub tag_anime: String,
	pub same_tag_anime: String,
	pub more_manga: String,
	pub same_manga: String,
	pub genre_manga: String,
	pub same_genre_manga: String,
	pub tag_manga: String,
	pub same_tag_manga: String,
	pub more_manga_chapter: String,
	pub same_manga_chapter: String,
}

use anyhow::Result;

pub async fn load_localization_compare(
	guild_id: String, db_config: DbConfig,
) -> Result<CompareLocalised> {
	let path = "json/message/anilist_user/compare.json";

	load_localization(guild_id, path, db_config).await
}
