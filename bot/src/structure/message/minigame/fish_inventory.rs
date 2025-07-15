use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;
use sea_orm::DatabaseConnection;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FishInventoryLocalised {
	pub title: String,
	pub empty_description: String,
	pub description: String,
	pub fish_by_type: String,
	pub best_specimens: String,
	pub rarity_distribution: String,
	pub summary: String,
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
	pub fish_format: String,
	pub specimen_format: String,
	pub rarity_format: String,
	pub total_fish: String,
	pub total_value: String,
}

pub async fn load_localization_fish_inventory(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<FishInventoryLocalised> {
	let path = "json/message/minigame/fish_inventory.json";

	load_localization(guild_id, path, db_connection).await
}