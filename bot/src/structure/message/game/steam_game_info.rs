use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct SteamGameInfoLocalised {
	pub field1: String,
	pub field2: String,
	pub field3: String,
	pub field4: String,
	pub field5: String,
	pub field6: String,
	pub field7: String,
	pub free: String,
	pub coming_soon: String,
	pub tba: String,
	pub win: String,
	pub mac: String,
	pub linux: String,
	pub website: String,
	pub required_age: String,
}

use anyhow::Result;

pub async fn load_localization_steam_game_info(
	guild_id: String, db_config: DbConfig,
) -> Result<SteamGameInfoLocalised> {
	let path = "json/message/game/steam_game_info.json";

	load_localization(guild_id, path, db_config).await
}
