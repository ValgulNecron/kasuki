use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LeaveLocalised {
	pub title: String,
	pub success: String,
}

use anyhow::Result;

pub async fn load_localization_leave(
	guild_id: String, db_config: DbConfig,
) -> Result<LeaveLocalised> {
	let path = "json/message/music/leave.json";

	load_localization(guild_id, path, db_config).await
}
