use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use kasuki_macros::slash_command;
use anyhow::{Context as AnyhowContext, Result};
use fluent_templates::fluent_bundle::FluentValue;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::item::{Entity as Item, Model as ItemModel};
use shared::database::user_inventory::ActiveModel as UserInventoryActiveModel;
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::{debug, info};

#[slash_command(
	name = "fishing", desc = "Go fishing!",
	command_type = SubCommand(parent = "minigame"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn fishing_command(self_: FishingCommand) -> Result<EmbedsContents<'_>> {
		self_.defer().await?;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

		let user_id = command_interaction.user.id.to_string();
		let server_id = command_interaction.guild_id.unwrap().to_string();

		// Get all fish items from the database
		let db_connection = bot_data.db_connection.clone();

		// Load localization data
		let lang_id = get_language_identifier(server_id.clone(), db_connection.clone()).await;

		let fish_items = get_fish_items(&db_connection).await?;

		if fish_items.is_empty() {
			return Err(anyhow::anyhow!("No fish items found in the database"));
		}

		// Generate a random fish based on weights
		let caught_fish = catch_random_fish(&fish_items)?;

		// Generate a random size for the fish (between 1 and 100)
		let fish_size = rand::rng().random_range(1..=100);

		// Generate a random rarity for the fish (between minimum_rarity and maximum_rarity)
		let fish_rarity = rand::rng().random_range(caught_fish.minimum_rarity..=caught_fish.maximum_rarity);

		// Add the fish to the user's inventory
		add_fish_to_inventory(
			&db_connection,
			user_id,
			server_id,
			caught_fish.item_id.clone(),
			fish_size,
			fish_rarity,
			caught_fish.base_xp_boost,
		).await?;

		// Create a message to display the caught fish
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

		// Format the fish details using the localized format string
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("fish_name"), FluentValue::from(caught_fish.name.clone()));
		args.insert(Cow::Borrowed("size_description"), FluentValue::from(size_description));
		args.insert(Cow::Borrowed("size"), FluentValue::from(fish_size.to_string()));
		args.insert(Cow::Borrowed("rarity"), FluentValue::from(rarity_text));
		args.insert(Cow::Borrowed("xp_boost"), FluentValue::from(((caught_fish.base_xp_boost * 100.0) as i32).to_string()));
		args.insert(Cow::Borrowed("price"), FluentValue::from(caught_fish.price.to_string()));

		let fish_details = USABLE_LOCALES.lookup_with_args(&lang_id, "minigame_fishing-fish_details_format", &args);

		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-title"))
			.fields(vec![
				(USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-caught_fish"), String::new(), false),
				(USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-fish_details"), fish_details, false),
				(USABLE_LOCALES.lookup(&lang_id, "minigame_fishing-description_field"), caught_fish.description.clone(), false),
			]);

		let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embeds_contents)
}

/// Get all fish items from the database
async fn get_fish_items(db: &DatabaseConnection) -> Result<Vec<ItemModel>> {
	debug!("Getting all fish items from the database");

	let fish_items = Item::find()
		.filter(shared::database::item::Column::Type.eq("fish"))
		.all(db)
		.await
		.context("Failed to get fish items from database")?;

	debug!("Found {} fish items", fish_items.len());

	Ok(fish_items)
}

/// Catch a random fish based on weights
fn catch_random_fish(fish_items: &[ItemModel]) -> Result<&ItemModel> {
	debug!("Generating random fish based on weights");

	// Create a weighted distribution based on the weights of the fish
	let weights: Vec<i32> = fish_items.iter().map(|item| item.weight).collect();
	let dist = WeightedIndex::new(&weights).context("Failed to create weighted distribution")?;

	// Generate a random index based on the weights
	let mut rng = rand::rng();
	let index = dist.sample(&mut rng);

	debug!("Selected fish: {}", fish_items[index].name);

	Ok(&fish_items[index])
}

/// Add a fish to the user's inventory
async fn add_fish_to_inventory(
	db: &DatabaseConnection, user_id: String, server_id: String, item_id: String, size: i32,
	rarity: i32, item_xp_boost: f32,
) -> Result<()> {
	debug!(
		"Adding fish to user's inventory: user_id={}, server_id={}, item_id={}",
		user_id, server_id, item_id
	);
	let unique_id = uuid::Uuid::new_v4().to_string();
	// Create a new inventory item
	let inventory_item = UserInventoryActiveModel {
		id: Set(unique_id),
		item_id: Set(item_id),
		user_id: Set(user_id),
		server_id: Set(server_id),
		size: Set(size),
		rarity: Set(rarity),
		item_xp_boost: Set(item_xp_boost),
	};

	// Insert the item into the database
	inventory_item
		.insert(db)
		.await
		.context("Failed to add fish to user's inventory")?;

	info!("Fish added to user's inventory successfully");

	Ok(())
}
