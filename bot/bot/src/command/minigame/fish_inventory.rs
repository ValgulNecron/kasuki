use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::{Context as AnyhowContext, Result};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
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
	name = "fish_inventory", desc = "Check your fish inventory.",
	command_type = SubCommand(parent = "minigame"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn fish_inventory_command(self_: FishInventoryCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();

	let user_id = command_interaction.user.id.to_string();
	let server_id = command_interaction.guild_id.unwrap().to_string();

	let db_connection = bot_data.db_connection.clone();

	let lang_id = get_language_identifier(server_id.clone(), db_connection.clone()).await;

	let inventory_items =
		get_user_fish_inventory(&db_connection, user_id.clone(), server_id.clone()).await?;

	if inventory_items.is_empty() {
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-title"))
				.description(
					USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-empty_description"),
				);

		let embeds_contents = EmbedsContents::new(vec![embed_content]);
		return Ok(embeds_contents);
	}

	let mut embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-description"));

	let mut fish_by_name: HashMap<String, Vec<(UserInventoryModel, ItemModel)>> = HashMap::new();

	for fish in &inventory_items {
		fish_by_name
			.entry(fish.1.name.clone())
			.or_insert_with(Vec::new)
			.push(fish.clone());
	}

	let mut fish_summary = String::new();

	for (fish_name, fish_list) in &fish_by_name {
		let count = fish_list.len();
		let total_value = fish_list.iter().map(|(_, item)| item.price).sum::<i32>();

		let base_rarity = match fish_list[0].1.minimum_rarity {
			1 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-common"),
			2 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-uncommon"),
			3 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-rare"),
			4 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-epic"),
			5 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-legendary"),
			_ => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-unknown"),
		};

		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(
			Cow::Borrowed("fish_name"),
			FluentValue::from(fish_name.clone()),
		);
		args.insert(Cow::Borrowed("base_rarity"), FluentValue::from(base_rarity));
		args.insert(Cow::Borrowed("count"), FluentValue::from(count.to_string()));
		args.insert(
			Cow::Borrowed("total_value"),
			FluentValue::from(total_value.to_string()),
		);

		fish_summary.push_str(&USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"minigame_fish_inventory-fish_format",
			&args,
		));
	}

	let mut fields = Vec::new();

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-fish_by_type"),
		fish_summary,
		false,
	));

	let mut fish_details = String::new();

	for (fish_name, fish_list) in &fish_by_name {
		if let Some((inventory_item, _)) = fish_list.iter().max_by(|a, b| {
			let rarity_cmp = a.0.rarity.cmp(&b.0.rarity);
			if rarity_cmp == std::cmp::Ordering::Equal {
				a.0.size.cmp(&b.0.size)
			} else {
				rarity_cmp
			}
		}) {
			let rarity_text = match inventory_item.rarity {
				1 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-common"),
				2 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-uncommon"),
				3 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-rare"),
				4 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-epic"),
				5 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-legendary"),
				_ => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-unknown"),
			};

			let size_description = match inventory_item.size {
				1..=20 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-tiny"),
				21..=40 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-small"),
				41..=60 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-average"),
				61..=80 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-large"),
				81..=95 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-huge"),
				96..=100 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-massive"),
				_ => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-unknown_size"),
			};

			let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
			args.insert(
				Cow::Borrowed("fish_name"),
				FluentValue::from(fish_name.clone()),
			);
			args.insert(
				Cow::Borrowed("size_description"),
				FluentValue::from(size_description),
			);
			args.insert(
				Cow::Borrowed("size"),
				FluentValue::from(inventory_item.size.to_string()),
			);
			args.insert(Cow::Borrowed("rarity"), FluentValue::from(rarity_text));
			args.insert(
				Cow::Borrowed("xp_boost"),
				FluentValue::from(((inventory_item.item_xp_boost * 100.0) as i32).to_string()),
			);

			fish_details.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_fish_inventory-specimen_format",
				&args,
			));
		}
	}

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-best_specimens"),
		fish_details,
		false,
	));

	let mut rarity_distribution_map = HashMap::new();
	for (inventory_item, _) in &inventory_items {
		*rarity_distribution_map
			.entry(inventory_item.rarity)
			.or_insert(0) += 1;
	}

	let mut rarity_text = String::new();
	for rarity in 1..=5 {
		let count = rarity_distribution_map.get(&rarity).unwrap_or(&0);
		let rarity_name = match rarity {
			1 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-common"),
			2 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-uncommon"),
			3 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-rare"),
			4 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-epic"),
			5 => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-legendary"),
			_ => USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-unknown"),
		};

		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("rarity_name"), FluentValue::from(rarity_name));
		args.insert(Cow::Borrowed("count"), FluentValue::from(count.to_string()));

		rarity_text.push_str(&USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"minigame_fish_inventory-rarity_format",
			&args,
		));
	}

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-rarity_distribution"),
		rarity_text,
		false,
	));

	let total_fish_count = inventory_items.len();
	let total_fish_value = inventory_items
		.iter()
		.map(|(_, item)| item.price)
		.sum::<i32>();

	let mut count_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	count_args.insert(
		Cow::Borrowed("count"),
		FluentValue::from(total_fish_count.to_string()),
	);

	let mut value_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	value_args.insert(
		Cow::Borrowed("value"),
		FluentValue::from(total_fish_value.to_string()),
	);

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "minigame_fish_inventory-summary"),
		format!(
			"{}\n{}",
			USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_fish_inventory-total_fish",
				&count_args
			),
			USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"minigame_fish_inventory-total_value",
				&value_args
			)
		),
		false,
	));

	embed_content = embed_content.fields(fields);

	let embeds_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embeds_contents)
}

/// Get a user's fish inventory with item details
async fn get_user_fish_inventory(
	db: &DatabaseConnection, user_id: String, server_id: String,
) -> Result<Vec<(UserInventoryModel, ItemModel)>> {
	debug!(
		"Getting fish inventory for user_id={}, server_id={}",
		user_id, server_id
	);

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
			if item.r#type == "fish" || (inventory_item.size > 0 && inventory_item.rarity > 0) {
				result.push((inventory_item, item));
			}
		}
	}

	debug!("Found {} fish items", result.len());

	Ok(result)
}
