use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueueLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub now_playing: String,
	pub nothing_playing: String,
	pub requested_by: String,
}

use anyhow::Result;

pub async fn load_localization_queue(
	guild_id: String, db_config: DbConfig,
) -> Result<QueueLocalised> {
	let path = "json/message/music/queue.json";

	load_localization(guild_id, path, db_config).await
}
