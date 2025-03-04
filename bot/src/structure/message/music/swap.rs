use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub error_same_index: String,
	pub error_max_index: String,
	pub success: String,
}

use anyhow::Result;

pub async fn load_localization_swap(
	guild_id: String, db_config: DbConfig,
) -> Result<SwapLocalised> {
	let path = "json/message/music/swap.json";

	load_localization(guild_id, path, db_config).await
}
