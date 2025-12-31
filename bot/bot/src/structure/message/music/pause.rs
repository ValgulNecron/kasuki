use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PauseLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub success: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_pause(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<PauseLocalised> {
	let path = "json/message/music/pause.json";

	load_localization(guild_id, path, db_connection).await
}
