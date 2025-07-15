use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::item::{Entity as Item, Model as ItemModel};
use crate::database::user_inventory::ActiveModel as UserInventoryActiveModel;
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::minigame::fishing::load_localization_fishing;
use anyhow::{Context as AnyhowContext, Result};
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, QueryFilter, Set};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::{debug, info};

#[derive(Clone)]
pub struct FishingCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
    for FishingCommand,
    get_contents = |self_: FishingCommand| async move {
        self_.defer().await;
        let ctx = self_.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = self_.get_command_interaction();

        let user_id = command_interaction.user.id.to_string();
        let server_id = command_interaction.guild_id.unwrap().to_string();

        // Get all fish items from the database
        let db_connection = bot_data.db_connection.clone();

        // Load localization data
        let localization = load_localization_fishing(
            server_id.clone(),
            db_connection.clone()
        ).await?;

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
            1 => localization.common.as_str(),
            2 => localization.uncommon.as_str(),
            3 => localization.rare.as_str(),
            4 => localization.epic.as_str(),
            5 => localization.legendary.as_str(),
            _ => localization.unknown.as_str(),
        };

        let size_description = match fish_size {
            1..=20 => localization.tiny.as_str(),
            21..=40 => localization.small.as_str(),
            41..=60 => localization.average.as_str(),
            61..=80 => localization.large.as_str(),
            81..=95 => localization.huge.as_str(),
            96..=100 => localization.massive.as_str(),
            _ => localization.unknown_size.as_str(),
        };

        // Format the fish details using the localized format string
        let fish_details = localization.fish_details_format
            .replace("$fish_name$", &caught_fish.name)
            .replace("$size_description$", size_description)
            .replace("$size$", &fish_size.to_string())
            .replace("$rarity$", rarity_text)
            .replace("$xp_boost$", &((caught_fish.base_xp_boost * 100.0) as i32).to_string())
            .replace("$price$", &caught_fish.price.to_string());

        let embed_content = EmbedContent::new(localization.title.clone())
            .fields(vec![
                (localization.caught_fish.clone(), String::new(), false),
                (localization.fish_details.clone(), fish_details, false),
                (localization.description_field.clone(), caught_fish.description.clone(), false),
            ]);

        let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

        Ok(embeds_contents)
    }
);

/// Get all fish items from the database
async fn get_fish_items(db: &DatabaseConnection) -> Result<Vec<ItemModel>> {
    debug!("Getting all fish items from the database");

    let fish_items = Item::find()
        .filter(crate::database::item::Column::Type.eq("fish"))
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
    db: &DatabaseConnection,
    user_id: String,
    server_id: String,
    item_id: String,
    size: i32,
    rarity: i32,
    item_xp_boost: f32,
) -> Result<()> {
    debug!("Adding fish to user's inventory: user_id={}, server_id={}, item_id={}", user_id, server_id, item_id);
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
    inventory_item.insert(db)
        .await
        .context("Failed to add fish to user's inventory")?;

    info!("Fish added to user's inventory successfully");

    Ok(())
}
