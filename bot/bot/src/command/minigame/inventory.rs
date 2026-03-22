use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::{Context as AnyhowContext, Result};
use kasuki_macros::slash_command;
use sea_orm::ExprTrait;
use sea_orm::{
	ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::item::{Entity as Item, Model as ItemModel};
use shared::database::user_inventory::{Entity as UserInventory, Model as UserInventoryModel};
use shared::fluent_args;
use shared::localization::{Loader, USABLE_LOCALES};
use std::collections::HashMap;

#[slash_command(
	name = "inventory", desc = "Check your inventory.",
	command_type = SubCommand(parent = "minigame"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn inventory_command(self_: InventoryCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let user_id = cx.command_interaction.user.id.to_string();

	let lang_id = cx.lang_id().await;

	let inventory_items = get_user_inventory(&cx.db, user_id.clone(), cx.guild_id.clone()).await?;

	if inventory_items.is_empty() {
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-title"))
				.description(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-empty"));

		let embeds_contents = EmbedsContents::new(vec![embed_content]);
		return Ok(embeds_contents);
	}

	let mut items_by_type: HashMap<String, Vec<(UserInventoryModel, ItemModel)>> = HashMap::new();

	for (inventory_item, item) in inventory_items {
		items_by_type
			.entry(item.r#type.clone())
			.or_insert_with(Vec::new)
			.push((inventory_item, item));
	}

	let mut embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-title"));

	if let Some(fish_items) = items_by_type.get("fish") {
		let mut fish_by_name: HashMap<String, Vec<(UserInventoryModel, ItemModel)>> =
			HashMap::new();

		for fish in fish_items {
			fish_by_name
				.entry(fish.1.name.clone())
				.or_insert_with(Vec::new)
				.push(fish.clone());
		}

		let mut fish_summary = String::new();

		for (fish_name, fish_list) in &fish_by_name {
			let count = fish_list.len();
			let total_value = fish_list.iter().map(|(_, item)| item.price).sum::<i32>();

			let args = fluent_args!(
				"fish_name" => fish_name.clone(),
				"count" => count.to_string(),
				"total_value" => total_value.to_string(),
			);

			fish_summary.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_inventory-fish_format",
				&args,
			));
			fish_summary.push('\n');
		}

		embed_content = embed_content.fields(vec![(
			USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-fish"),
			fish_summary,
			false,
		)]);

		let mut fish_details = String::new();
		let mut fish_count = 0;

		let mut indices: Vec<usize> = (0..fish_items.len()).collect();
		indices.sort_by(|&a, &b| {
			let rarity_cmp = fish_items[b].0.rarity.cmp(&fish_items[a].0.rarity);
			if rarity_cmp == std::cmp::Ordering::Equal {
				fish_items[b].0.size.cmp(&fish_items[a].0.size)
			} else {
				rarity_cmp
			}
		});

		for &idx in indices.iter().take(10) {
			let (inventory_item, item) = &fish_items[idx];
			fish_count += 1;

			let rarity_text = match inventory_item.rarity {
				1 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-common"),
				2 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-uncommon"),
				3 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-rare"),
				4 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-epic"),
				5 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-legendary"),
				_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown"),
			};

			let size_description = match inventory_item.size {
				1..=20 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-tiny"),
				21..=40 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-small"),
				41..=60 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-average"),
				61..=80 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-large"),
				81..=95 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-huge"),
				96..=100 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-massive"),
				_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown_size"),
			};

			let args = fluent_args!(
				"count" => fish_count.to_string(),
				"item_name" => item.name.clone(),
				"size_description" => size_description,
				"size" => inventory_item.size.to_string(),
				"rarity_text" => rarity_text,
				"xp_boost" => ((inventory_item.item_xp_boost * 100.0) as i32).to_string(),
			);

			fish_details.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_inventory-best_fish_format",
				&args,
			));
			fish_details.push('\n');
		}

		embed_content = embed_content.fields(vec![(
			USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-best_fish"),
			fish_details,
			false,
		)]);

		let mut all_fish_list = String::new();
		let mut fish_number = 1;

		let mut name_indices: Vec<usize> = (0..fish_items.len()).collect();
		name_indices.sort_by(|&a, &b| fish_items[a].1.name.cmp(&fish_items[b].1.name));

		for &idx in &name_indices {
			let (inventory_item, item) = &fish_items[idx];
			let rarity_text = match inventory_item.rarity {
				1 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-common"),
				2 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-uncommon"),
				3 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-rare"),
				4 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-epic"),
				5 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-legendary"),
				_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown"),
			};

			let size_description = match inventory_item.size {
				1..=20 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-tiny"),
				21..=40 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-small"),
				41..=60 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-average"),
				61..=80 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-large"),
				81..=95 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-huge"),
				96..=100 => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-massive"),
				_ => USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-unknown_size"),
			};

			let args = fluent_args!(
				"fish_number" => fish_number.to_string(),
				"item_name" => item.name.clone(),
				"size_description" => size_description,
				"size" => inventory_item.size.to_string(),
				"rarity_text" => rarity_text,
				"price" => item.price.to_string(),
			);

			all_fish_list.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_inventory-all_fish_format",
				&args,
			));
			all_fish_list.push('\n');

			fish_number += 1;
		}

		embed_content = embed_content.fields(vec![(
			USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-all_fish"),
			all_fish_list,
			false,
		)]);

		let total_fish_count = fish_items.len();
		let total_fish_value = fish_items.iter().map(|(_, item)| item.price).sum::<i32>();

		let summary_args = fluent_args!(
			"total_fish_count" => total_fish_count.to_string(),
			"total_fish_value" => total_fish_value.to_string(),
		);

		embed_content = embed_content.fields(vec![(
			USABLE_LOCALES.lookup(&lang_id, "minigame_inventory-summary"),
			USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_inventory-summary_format",
				&summary_args,
			),
			false,
		)]);
	}

	let embeds_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embeds_contents)
}

async fn get_user_inventory(
	db: &DatabaseConnection, user_id: String, server_id: String,
) -> Result<Vec<(UserInventoryModel, ItemModel)>> {
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
