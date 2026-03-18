use anyhow::{Context, Result};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use shared::database::item::{ActiveModel as ItemActiveModel, Entity as Item};
use shared::helper::read_file::read_file_as_string;

#[derive(Debug, Serialize, Deserialize)]
struct ItemsJson {
	items: Vec<ItemJson>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ItemJson {
	item_id: String,
	name: String,
	description: String,
	price: i32,
	minimum_rarity: i32,
	maximum_rarity: i32,
	r#type: String,
	base_xp_boost: f32,
	weight: i32,
}

/// Loads items from a JSON file and inserts them into the database.
///
/// # Arguments
///
/// * `db` - The database connection
///
/// # Returns
///
/// A Result indicating success or failure
pub async fn load_items_from_json(db: &DatabaseConnection) -> Result<()> {
	info!("Loading items from JSON file");

	// Read the JSON file
	let json_content =
		read_file_as_string("./json/items/items.json").context("Failed to read items JSON file")?;

	// Parse the JSON
	let items_json: ItemsJson =
		serde_json::from_str(&json_content).context("Failed to parse items JSON")?;

	debug!("Found {} items in JSON file", items_json.items.len());

	// Insert each item into the database
	for item_json in items_json.items {
		debug!("Processing item: {}", item_json.item_id);

		// Check if the item already exists
		let existing_item = Item::find_by_id(item_json.item_id.clone())
			.one(db)
			.await
			.context("Failed to check if item exists")?;

		if existing_item.is_some() {
			debug!("Item already exists: {}", item_json.item_id);
			continue;
		}

		// Create a new item
		let item = ItemActiveModel {
			item_id: Set(item_json.item_id.clone()),
			name: Set(item_json.name),
			description: Set(item_json.description),
			price: Set(item_json.price),
			minimum_rarity: Set(item_json.minimum_rarity),
			maximum_rarity: Set(item_json.maximum_rarity),
			r#type: Set(item_json.r#type),
			base_xp_boost: Set(item_json.base_xp_boost),
			weight: Set(item_json.weight),
		};

		// Insert the item into the database
		item.insert(db)
			.await
			.context(format!("Failed to insert item: {}", item_json.item_id))?;

		info!("Inserted item: {}", item_json.item_id);
	}

	info!("Finished loading items from JSON file");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_items_json() {
		// CARGO_MANIFEST_DIR points to bot/bot/, the json lives at bot/json/
		let path = format!("{}/../json/items/items.json", env!("CARGO_MANIFEST_DIR"));
		let json_content = read_file_as_string(&path).unwrap();
		let items_json: ItemsJson = serde_json::from_str(&json_content).unwrap();

		assert!(!items_json.items.is_empty());

		// Check that the first item has the expected fields
		let first_item = &items_json.items[0];
		assert_eq!(first_item.item_id, "minigame_coin");
		assert_eq!(first_item.name, "Minigame Coin");
		assert_eq!(first_item.r#type, "minigame");
	}
}
