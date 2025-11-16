use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct GameLocalised {
	pub released: String,

	pub platforms: String,

	pub playtime: String,

	pub tags: String,

	pub developers: String,

	pub staff: String,

	pub characters: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_game(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<GameLocalised> {
	let path = "json/message/vn/game.json";

	load_localization(guild_id, path, db_connection).await
}
