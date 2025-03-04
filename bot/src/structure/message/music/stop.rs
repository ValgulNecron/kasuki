use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StopLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub success: String,
	pub nothing_to_stop: String,
}

use anyhow::Result;

pub async fn load_localization_stop(
	guild_id: String, db_config: DbConfig,
) -> Result<StopLocalised> {
	let path = "json/message/music/stop.json";

	load_localization(guild_id, path, db_config).await
}
