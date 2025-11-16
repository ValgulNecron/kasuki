use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct LevelLocalised {
	pub desc: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_level(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<LevelLocalised> {
	let path = "json/message/anilist_user/level.json";

	load_localization(guild_id, path, db_connection).await
}
