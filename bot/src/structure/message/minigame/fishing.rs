use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;
use sea_orm::DatabaseConnection;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FishingLocalised {
	pub title: String,
	pub caught_fish: String,
	pub fish_details: String,
	pub description_field: String,
	pub common: String,
	pub uncommon: String,
	pub rare: String,
	pub epic: String,
	pub legendary: String,
	pub unknown: String,
	pub tiny: String,
	pub small: String,
	pub average: String,
	pub large: String,
	pub huge: String,
	pub massive: String,
	pub unknown_size: String,
	pub fish_details_format: String,
}

pub async fn load_localization_fishing(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<FishingLocalised> {
	let path = "json/message/minigame/fishing.json";

	load_localization(guild_id, path, db_connection).await
}