use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct RandomImageLocalised {
	pub title: String,
}

use anyhow::Result;

pub async fn load_localization_random_image(
	guild_id: String, db_config: DbConfig,
) -> Result<RandomImageLocalised> {
	let path = "json/message/anime/random_image.json";

	load_localization(guild_id, path, db_config).await
}
