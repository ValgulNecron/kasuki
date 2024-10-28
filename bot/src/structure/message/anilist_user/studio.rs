use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct StudioLocalised {
	pub desc: String,
}

use anyhow::Result;

pub async fn load_localization_studio(
	guild_id: String, db_config: DbConfig,
) -> Result<StudioLocalised> {
	let path = "json/message/anilist_user/studio.json";

	load_localization(guild_id, path, db_config).await
}
