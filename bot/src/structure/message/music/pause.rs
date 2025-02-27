use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PauseLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub success: String,
}

use anyhow::Result;

pub async fn load_localization_pause(
	guild_id: String, db_config: DbConfig,
) -> Result<PauseLocalised> {
	let path = "json/message/music/pause.json";

	load_localization(guild_id, path, db_config).await
}
