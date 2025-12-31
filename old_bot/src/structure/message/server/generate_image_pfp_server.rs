use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct PFPServerLocalisedImage {
	pub title: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_pfp_server_image(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<PFPServerLocalisedImage> {
	let path = "json/message/server/generate_image_pfp_server.json";

	load_localization(guild_id, path, db_connection).await
}
