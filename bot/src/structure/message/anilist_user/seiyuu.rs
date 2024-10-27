use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct SeiyuuLocalised {
	pub title: String,
}

use anyhow::Result;

pub async fn load_localization_seiyuu(
	guild_id: String, db_config: DbConfig,
) -> Result<SeiyuuLocalised> {
	let path = "json/message/anilist_user/seiyuu.json";

	load_localization(guild_id, path, db_config).await
}
