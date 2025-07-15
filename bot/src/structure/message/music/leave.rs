use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LeaveLocalised {
	pub title: String,
	pub success: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_leave(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<LeaveLocalised> {
	let path = "json/message/music/leave.json";

	load_localization(guild_id, path, db_connection).await
}
