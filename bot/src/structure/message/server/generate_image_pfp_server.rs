use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct PFPServerLocalisedImage {
	pub title: String,
}

use anyhow::Result;

pub async fn load_localization_pfp_server_image(
	guild_id: String, db_config: DbConfig,
) -> Result<PFPServerLocalisedImage> {
	let path = "json/message/server/generate_image_pfp_server.json";

	load_localization(guild_id, path, db_config).await
}
