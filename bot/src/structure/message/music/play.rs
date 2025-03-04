use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub added_to_queue: String,
	pub added_playlist: String,
}

use anyhow::Result;

pub async fn load_localization_play(
	guild_id: String, db_config: DbConfig,
) -> Result<PlayLocalised> {
	let path = "json/message/music/play.json";

	load_localization(guild_id, path, db_config).await
}
