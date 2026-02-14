use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use kasuki_macros::slash_command;
use anyhow::{Context as AnyhowContext, Result};
use fluent_templates::fluent_bundle::FluentValue;
use sea_orm::ExprTrait;
use sea_orm::{
	ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::item::{Entity as Item, Model as ItemModel};
use shared::database::user_inventory::{Entity as UserInventory, Model as UserInventoryModel};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::debug;

#[slash_command(
	name = "inventory", desc = "Check your inventory.",
	command_type = SubCommand(parent = "minigame"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn inventory_command(self_: InventoryCommand) -> Result<EmbedsContents<'_>> {
		self_.defer().await?;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

		let user_id = command_interaction.user.id.to_string();
		let server_id = command_interaction.guild_id.unwrap().to_string();

		// Get the user's inventory
		let db_connection = bot_data.db_connection.clone();

		// Load localization data
		let lang_id = get_language_identifier(server_id.clone(), db_connection.clone()).await;

		let inventory_items = get_user_inventory(&db_connection, user_id.clone(), server_id.clone()).await?;

		if inventory_items.is_empty() {
			// Create an embed message for empty inventory
			let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-title"))
				.description(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-empty"));

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
		let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-title"));

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

			for (fish_name, fish_list) in &fish_by_name {
				let count = fish_list.len();
				let total_value = fish_list.iter().map(|(_, item)| item.price).sum::<i32>();

				let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
				args.insert(Cow::Borrowed("fish_name"), FluentValue::from(fish_name.clone()));
				args.insert(Cow::Borrowed("count"), FluentValue::from(count.to_string()));
				args.insert(Cow::Borrowed("total_value"), FluentValue::from(total_value.to_string()));

				fish_summary.push_str(&USABLE_LOCALES.lookup_with_args(&lang_id, "minigame_inventory-fish_format", &args));
				fish_summary.push('\n');
			}

			// Add fish summary to the embed
			embed_content = embed_content.fields(vec![
				(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-fish"), fish_summary, false),
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
					1 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-common"),
					2 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-uncommon"),
					3 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-rare"),
					4 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-epic"),
					5 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-legendary"),
					_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown"),
				};

				// Get size description
				let size_description = match inventory_item.size {
					1..=20 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-tiny"),
					21..=40 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-small"),
					41..=60 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-average"),
					61..=80 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-large"),
					81..=95 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-huge"),
					96..=100 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-massive"),
					_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown_size"),
				};

				let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
				args.insert(Cow::Borrowed("count"), FluentValue::from(fish_count.to_string()));
				args.insert(Cow::Borrowed("item_name"), FluentValue::from(item.name.clone()));
				args.insert(Cow::Borrowed("size_description"), FluentValue::from(size_description));
				args.insert(Cow::Borrowed("size"), FluentValue::from(inventory_item.size.to_string()));
				args.insert(Cow::Borrowed("rarity_text"), FluentValue::from(rarity_text));
				args.insert(Cow::Borrowed("xp_boost"), FluentValue::from(((inventory_item.item_xp_boost * 100.0) as i32).to_string()));

				fish_details.push_str(&USABLE_LOCALES.lookup_with_args(&lang_id, "minigame_inventory-best_fish_format", &args));
				fish_details.push('\n');
			}

			// Add fish details to the embed
			embed_content = embed_content.fields(vec![
				(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-best_fish"), fish_details, false),
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
					1 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-common"),
					2 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-uncommon"),
					3 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-rare"),
					4 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-epic"),
					5 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-legendary"),
					_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown"),
				};

				// Get size description
				let size_description = match inventory_item.size {
					1..=20 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-tiny"),
					21..=40 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-small"),
					41..=60 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-average"),
					61..=80 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-large"),
					81..=95 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-huge"),
					96..=100 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-massive"),
					_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown_size"),
				};

				let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
				args.insert(Cow::Borrowed("fish_number"), FluentValue::from(fish_number.to_string()));
				args.insert(Cow::Borrowed("item_name"), FluentValue::from(item.name.clone()));
				args.insert(Cow::Borrowed("size_description"), FluentValue::from(size_description));
				args.insert(Cow::Borrowed("size"), FluentValue::from(inventory_item.size.to_string()));
				args.insert(Cow::Borrowed("rarity_text"), FluentValue::from(rarity_text));
				args.insert(Cow::Borrowed("price"), FluentValue::from(item.price.to_string()));

				all_fish_list.push_str(&USABLE_LOCALES.lookup_with_args(&lang_id, "minigame_inventory-all_fish_format", &args));
				all_fish_list.push('\n');

				fish_number += 1;
			}

			// Add the complete fish list to the embed
			embed_content = embed_content.fields(vec![
				(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-all_fish"), all_fish_list, false),
			]);

			// Add total count and value
			let total_fish_count = fish_items.len();
			let total_fish_value = fish_items.iter().map(|(_, item)| item.price).sum::<i32>();

			let mut summary_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
			summary_args.insert(Cow::Borrowed("total_fish_count"), FluentValue::from(total_fish_count.to_string()));
			summary_args.insert(Cow::Borrowed("total_fish_value"), FluentValue::from(total_fish_value.to_string()));

			embed_content = embed_content.fields(vec![
				(
					USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-summary"),
					USABLE_LOCALES.lookup_with_args(&lang_id, "minigame_inventory-summary_format", &summary_args),
					false
				),
			]);
		}

		// Add other item types if needed in the future

		let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embeds_contents)
}

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
			shared::database::user_inventory::Column::UserId
				.eq(user_id.clone())
				.and(shared::database::user_inventory::Column::ServerId.eq(server_id.clone())),
		)
		.join(
			JoinType::InnerJoin,
			shared::database::user_inventory::Relation::Item.def(),
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
