use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::{Context as AnyhowContext, Result};
use kasuki_macros::slash_command;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::item::{Entity as Item, Model as ItemModel};
use shared::database::user_inventory::ActiveModel as UserInventoryActiveModel;
use shared::fluent_args;
use shared::localization::{Loader, USABLE_LOCALES};
use tracing::info;

#[slash_command(
	name = "fishing", desc = "Go fishing!",
	command_type = SubCommand(parent = "minigame"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn fishing_command(self_: FishingCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let user_id = cx.command_interaction.user.id.to_string();

	let lang_id = cx.lang_id().await;

	let fish_items = get_fish_items(&cx.db).await?;

	if fish_items.is_empty() {
		return Err(anyhow::anyhow!("No fish items found in the database"));
	}

	let caught_fish = catch_random_fish(&fish_items)?;

	let fish_size = rand::rng().random_range(1..=100);

	let fish_rarity =
		rand::rng().random_range(caught_fish.minimum_rarity..=caught_fish.maximum_rarity);

	add_fish_to_inventory(
		&cx.db,
		user_id,
		cx.guild_id.clone(),
		caught_fish.item_id.clone(),
		fish_size,
		fish_rarity,
		caught_fish.base_xp_boost,
	)
	.await?;

	let rarity_text = match fish_rarity {
		1 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-common"),
		2 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-uncommon"),
		3 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-rare"),
		4 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-epic"),
		5 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-legendary"),
		_ => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-unknown"),
	};

	let size_description = match fish_size {
		1..=20 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-tiny"),
		21..=40 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-small"),
		41..=60 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-average"),
		61..=80 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-large"),
		81..=95 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-huge"),
		96..=100 => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-massive"),
		_ => USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-unknown_size"),
	};

	let args = fluent_args!(
		"fish_name" => caught_fish.name.clone(),
		"size_description" => size_description,
		"size" => fish_size.to_string(),
		"rarity" => rarity_text,
		"xp_boost" => ((caught_fish.base_xp_boost * 100.0) as i32).to_string(),
		"price" => caught_fish.price.to_string(),
	);

	let fish_details =
		USABLE_LOCALES.lookup_with_args(&lang_id, "minigame_fishing-fish_details_format", &args);

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-title")).fields(vec![
			(
				USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-caught_fish"),
				String::new(),
				false,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-fish_details"),
				fish_details,
				false,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-description_field"),
				caught_fish.description.clone(),
				false,
			),
		]);

	let embeds_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embeds_contents)
}

async fn get_fish_items(db: &DatabaseConnection) -> Result<Vec<ItemModel>> {
	let fish_items = Item::find()
		.filter(shared::database::item::Column::Type.eq("fish"))
		.all(db)
		.await
		.context("Failed to get fish items from database")?;

	Ok(fish_items)
}

fn catch_random_fish(fish_items: &[ItemModel]) -> Result<&ItemModel> {
	let weights: Vec<i32> = fish_items.iter().map(|item| item.weight).collect();
	let dist = WeightedIndex::new(&weights).context("Failed to create weighted distribution")?;

	let mut rng = rand::rng();
	let index = dist.sample(&mut rng);

	Ok(&fish_items[index])
}

async fn add_fish_to_inventory(
	db: &DatabaseConnection, user_id: String, server_id: String, item_id: String, size: i32,
	rarity: i32, item_xp_boost: f32,
) -> Result<()> {
	let unique_id = uuid::Uuid::new_v4().to_string();
	let inventory_item = UserInventoryActiveModel {
		id: Set(unique_id),
		item_id: Set(item_id),
		user_id: Set(user_id),
		server_id: Set(server_id),
		size: Set(size),
		rarity: Set(rarity),
		item_xp_boost: Set(item_xp_boost),
	};

	inventory_item
		.insert(db)
		.await
		.context("Failed to add fish to user's inventory")?;

	info!("Fish added to user's inventory successfully");

	Ok(())
}
