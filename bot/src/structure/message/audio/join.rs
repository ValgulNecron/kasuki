#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct JoinLocalised {
	pub title: String,

	pub already_in: String,
}

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub async fn load_localization_join_localised(
	guild_id: String, db_config: DbConfig,
) -> Result<JoinLocalised> {
	let path = "json/message/audio/join.json";

	load_localization(guild_id, path, db_config).await
}
