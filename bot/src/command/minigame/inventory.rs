use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::item::{Entity as Item, Model as ItemModel};
use crate::database::user_inventory::{Entity as UserInventory, Model as UserInventoryModel};
use crate::event_handler::BotData;
use crate::impl_command;
use anyhow::{Context as AnyhowContext, Result};
use sea_orm::{
	ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::collections::HashMap;
use tracing::debug;

#[derive(Clone)]
pub struct InventoryCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for InventoryCommand,
	get_contents = |self_: InventoryCommand| async move {
		self_.defer().await;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

		let user_id = command_interaction.user.id.to_string();
		let server_id = command_interaction.guild_id.unwrap().to_string();

		// Get the user's inventory
		let db_connection = bot_data.db_connection.clone();
		let inventory_items = get_user_inventory(&db_connection, user_id.clone(), server_id.clone()).await?;

		if inventory_items.is_empty() {
			// Create an embed message for empty inventory
			let embed_content = EmbedContent::new("Your Inventory".to_string())
				.description("Your inventory is empty. Try using the `/minigame fishing` command to catch some fish!".to_string());

			let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
			return Ok(embeds_contents);
		}

		// Group items by type
		let mut items_by_type: HashMap<String, Vec<(UserInventoryModel, ItemModel)>> = HashMap::new();

		for (inventory_item, item) in inventory_items {
			items_by_type
				.entry(item.r#type.clone())
				.or_insert_with(Vec::new)
				.push((inventory_item, item));
		}

		// Create an embed message for the inventory
		let mut embed_content = EmbedContent::new("Your Inventory".to_string());

		// Add fish section if the user has fish
		if let Some(fish_items) = items_by_type.get("fish") {
			// Group fish by name
			let mut fish_by_name: HashMap<String, Vec<(UserInventoryModel, ItemModel)>> = HashMap::new();

			for fish in fish_items {
				fish_by_name
					.entry(fish.1.name.clone())
					.or_insert_with(Vec::new)
					.push(fish.clone());
			}

			// Create a summary of fish
			let mut fish_summary = String::new();

			for (fish_name, fish_list) in fish_by_name {
				let count = fish_list.len();
				let total_value = fish_list.iter().map(|(_, item)| item.price).sum::<i32>();

				fish_summary.push_str(&format!(
					"**{}** x{} (Total Value: {} coins)\n",
					fish_name,
					count,
					total_value
				));
			}

			// Add fish summary to the embed
			embed_content = embed_content.fields(vec![
				("Fish".to_string(), fish_summary, false),
			]);

			// Add fish details section
			let mut fish_details = String::new();
			let mut fish_count = 0;

			// Sort fish by rarity (highest to lowest) and size (largest to smallest)
			let mut sorted_fish = fish_items.clone();
			sorted_fish.sort_by(|a, b| {
				let rarity_cmp = b.0.rarity.cmp(&a.0.rarity);
				if rarity_cmp == std::cmp::Ordering::Equal {
					b.0.size.cmp(&a.0.size)
				} else {
					rarity_cmp
				}
			});

			// Show only the top 10 fish
			for (inventory_item, item) in sorted_fish.iter().take(10) {
				fish_count += 1;

				// Get rarity text
				let rarity_text = match inventory_item.rarity {
					1 => "Common",
					2 => "Uncommon",
					3 => "Rare",
					4 => "Epic",
					5 => "Legendary",
					_ => "Unknown",
				};

				// Get size description
				let size_description = match inventory_item.size {
					1..=20 => "tiny",
					21..=40 => "small",
					41..=60 => "average",
					61..=80 => "large",
					81..=95 => "huge",
					96..=100 => "massive",
					_ => "unknown",
				};

				fish_details.push_str(&format!(
					"**{}. {}** - {} ({} cm), {} rarity, {}% XP boost\n",
					fish_count,
					item.name,
					size_description,
					inventory_item.size,
					rarity_text,
					(inventory_item.item_xp_boost * 100.0) as i32
				));
			}

			// Add fish details to the embed
			embed_content = embed_content.fields(vec![
				("Your Best Fish".to_string(), fish_details, false),
			]);

			// Add a complete numbered list of all fish
			let mut all_fish_list = String::new();
			let mut fish_number = 1;

			// Sort all fish by name for a consistent display
			let mut all_fish = fish_items.clone();
			all_fish.sort_by(|a, b| a.1.name.cmp(&b.1.name));

			for (inventory_item, item) in &all_fish {
				// Get rarity text
				let rarity_text = match inventory_item.rarity {
					1 => "Common",
					2 => "Uncommon",
					3 => "Rare",
					4 => "Epic",
					5 => "Legendary",
					_ => "Unknown",
				};

				// Get size description
				let size_description = match inventory_item.size {
					1..=20 => "tiny",
					21..=40 => "small",
					41..=60 => "average",
					61..=80 => "large",
					81..=95 => "huge",
					96..=100 => "massive",
					_ => "unknown",
				};

				all_fish_list.push_str(&format!(
					"**{}. {}** - {} ({} cm), {} rarity, Value: {} coins\n",
					fish_number,
					item.name,
					size_description,
					inventory_item.size,
					rarity_text,
					item.price
				));

				fish_number += 1;
			}

			// Add the complete fish list to the embed
			embed_content = embed_content.fields(vec![
				("All Fish (Numbered)".to_string(), all_fish_list, false),
			]);

			// Add total count and value
			let total_fish_count = fish_items.len();
			let total_fish_value = fish_items.iter().map(|(_, item)| item.price).sum::<i32>();

			embed_content = embed_content.fields(vec![
				(
					"Summary".to_string(),
					format!(
						"Total Fish: {}\nTotal Value: {} coins",
						total_fish_count,
						total_fish_value
					),
					false
				),
			]);
		}

		// Add other item types if needed in the future

		let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embeds_contents)
	}
);

/// Get a user's inventory with item details
async fn get_user_inventory(
	db: &DatabaseConnection, user_id: String, server_id: String,
) -> Result<Vec<(UserInventoryModel, ItemModel)>> {
	debug!(
		"Getting inventory for user_id={}, server_id={}",
		user_id, server_id
	);

	// Query the user's inventory with a join to the item table
	let inventory_items = UserInventory::find()
		.filter(
			crate::database::user_inventory::Column::UserId
				.eq(user_id.clone())
				.and(crate::database::user_inventory::Column::ServerId.eq(server_id.clone())),
		)
		.join(
			JoinType::InnerJoin,
			crate::database::user_inventory::Relation::Item.def(),
		)
		.all(db)
		.await
		.context("Failed to get user inventory from database")?;

	debug!("Found {} inventory items", inventory_items.len());

	// Get the item details for each inventory item
	let mut result = Vec::new();

	for inventory_item in inventory_items {
		let item = Item::find_by_id(inventory_item.item_id.clone())
			.one(db)
			.await
			.context(format!(
				"Failed to get item details for item_id={}",
				inventory_item.item_id
			))?;

		if let Some(item) = item {
			result.push((inventory_item, item));
		}
	}

	Ok(result)
}
