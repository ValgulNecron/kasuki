use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct RegisterLocalised {
	pub desc: String,
}

use anyhow::Result;

pub async fn load_localization_register(
	guild_id: String, db_config: DbConfig,
) -> Result<RegisterLocalised> {
	let path = "json/message/anilist_user/register.json";

	load_localization(guild_id, path, db_config).await
}
