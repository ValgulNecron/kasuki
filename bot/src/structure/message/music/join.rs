use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JoinLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub error_joining: String,
	pub success: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_join(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<JoinLocalised> {
	let path = "json/message/music/join.json";

	load_localization(guild_id, path, db_connection).await
}
