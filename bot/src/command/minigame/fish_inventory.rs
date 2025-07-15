use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::item::{Entity as Item, Model as ItemModel};
use crate::database::user_inventory::{Entity as UserInventory, Model as UserInventoryModel};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::minigame::fish_inventory::load_localization_fish_inventory;
use anyhow::{Context as AnyhowContext, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::collections::HashMap;
use tracing::debug;

#[derive(Clone)]
pub struct FishInventoryCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
    for FishInventoryCommand,
    get_contents = |self_: FishInventoryCommand| async move {
        self_.defer().await;
        let ctx = self_.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = self_.get_command_interaction();

        let user_id = command_interaction.user.id.to_string();
        let server_id = command_interaction.guild_id.unwrap().to_string();

        // Get the user's inventory
        let db_connection = bot_data.db_connection.clone();

        // Load localization data
        let localization = load_localization_fish_inventory(
            server_id.clone(),
            db_connection.clone()
        ).await?;

        let inventory_items = get_user_fish_inventory(&db_connection, user_id.clone(), server_id.clone()).await?;

        if inventory_items.is_empty() {
            // Create an embed message for empty inventory
            let embed_content = EmbedContent::new(localization.title.clone())
                .description(localization.empty_description.clone());

            let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
            return Ok(embeds_contents);
        }

        // Create an embed message for the fish inventory
        let mut embed_content = EmbedContent::new(localization.title.clone())
            .description(localization.description.clone());

        // Group fish by name
        let mut fish_by_name: HashMap<String, Vec<(UserInventoryModel, ItemModel)>> = HashMap::new();

        for fish in &inventory_items {
            fish_by_name
                .entry(fish.1.name.clone())
                .or_insert_with(Vec::new)
                .push(fish.clone());
        }

        // Create a summary of fish by type
        let mut fish_summary = String::new();

        for (fish_name, fish_list) in &fish_by_name {
            let count = fish_list.len();
            let total_value = fish_list.iter().map(|(_, item)| item.price).sum::<i32>();

            // Get the rarity of the first fish (they all have the same base rarity)
            let base_rarity = match fish_list[0].1.minimum_rarity {
                1 => localization.common.as_str(),
                2 => localization.uncommon.as_str(),
                3 => localization.rare.as_str(),
                4 => localization.epic.as_str(),
                5 => localization.legendary.as_str(),
                _ => localization.unknown.as_str(),
            };

            fish_summary.push_str(
                &localization.fish_format
                    .replace("$fish_name$", fish_name)
                    .replace("$base_rarity$", base_rarity)
                    .replace("$count$", &count.to_string())
                    .replace("$total_value$", &total_value.to_string())
            );
        }

        // Create a vector to hold all fields
        let mut fields = Vec::new();

        // Add fish summary to the embed
        fields.push((localization.fish_by_type.clone(), fish_summary, false));

        // Add fish details section - show best specimens of each type
        let mut fish_details = String::new();

        // Process each type of fish
        for (fish_name, fish_list) in &fish_by_name {
            // Sort this type of fish by rarity (highest to lowest) and size (largest to smallest)
            let mut sorted_fish = fish_list.clone();
            sorted_fish.sort_by(|a, b| {
                let rarity_cmp = b.0.rarity.cmp(&a.0.rarity);
                if rarity_cmp == std::cmp::Ordering::Equal {
                    b.0.size.cmp(&a.0.size)
                } else {
                    rarity_cmp
                }
            });

            // Show only the best specimen of each type
            if let Some((inventory_item, item)) = sorted_fish.first() {
                // Get rarity text
                let rarity_text = match inventory_item.rarity {
                    1 => localization.common.as_str(),
                    2 => localization.uncommon.as_str(),
                    3 => localization.rare.as_str(),
                    4 => localization.epic.as_str(),
                    5 => localization.legendary.as_str(),
                    _ => localization.unknown.as_str(),
                };

                // Get size description
                let size_description = match inventory_item.size {
                    1..=20 => localization.tiny.as_str(),
                    21..=40 => localization.small.as_str(),
                    41..=60 => localization.average.as_str(),
                    61..=80 => localization.large.as_str(),
                    81..=95 => localization.huge.as_str(),
                    96..=100 => localization.massive.as_str(),
                    _ => localization.unknown_size.as_str(),
                };

                fish_details.push_str(
                    &localization.specimen_format
                        .replace("$fish_name$", fish_name)
                        .replace("$size_description$", size_description)
                        .replace("$size$", &inventory_item.size.to_string())
                        .replace("$rarity$", rarity_text)
                        .replace("$xp_boost$", &((inventory_item.item_xp_boost * 100.0) as i32).to_string())
                );
            }
        }

        // Add fish details to the embed
        fields.push((localization.best_specimens.clone(), fish_details, false));

        // Sort all fish by name for a consistent display
        let mut all_fish = inventory_items.clone();
        all_fish.sort_by(|a, b| a.1.name.cmp(&b.1.name));

        // Add rarity distribution section
        let mut rarity_distribution = HashMap::new();
        for (inventory_item, _) in &inventory_items {
            *rarity_distribution.entry(inventory_item.rarity).or_insert(0) += 1;
        }

        let mut rarity_text = String::new();
        for rarity in 1..=5 {
            let count = rarity_distribution.get(&rarity).unwrap_or(&0);
            let rarity_name = match rarity {
                1 => localization.common.as_str(),
                2 => localization.uncommon.as_str(),
                3 => localization.rare.as_str(),
                4 => localization.epic.as_str(),
                5 => localization.legendary.as_str(),
                _ => localization.unknown.as_str(),
            };

            rarity_text.push_str(
                &localization.rarity_format
                    .replace("$rarity_name$", rarity_name)
                    .replace("$count$", &count.to_string())
            );
        }

        // Add rarity distribution to the embed
        fields.push((localization.rarity_distribution.clone(), rarity_text, false));

        // Add total count and value
        let total_fish_count = inventory_items.len();
        let total_fish_value = inventory_items.iter().map(|(_, item)| item.price).sum::<i32>();

        fields.push((
            localization.summary.clone(),
            format!(
                "{}\n{}",
                localization.total_fish.replace("$count$", &total_fish_count.to_string()),
                localization.total_value.replace("$value$", &total_fish_value.to_string())
            ),
            false
        ));

        // Add all fields to the embed content
        embed_content = embed_content.fields(fields);

        let embeds_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

        Ok(embeds_contents)
    }
);

/// Get a user's fish inventory with item details
async fn get_user_fish_inventory(
    db: &DatabaseConnection,
    user_id: String,
    server_id: String,
) -> Result<Vec<(UserInventoryModel, ItemModel)>> {
    debug!("Getting fish inventory for user_id={}, server_id={}", user_id, server_id);

    // Query the user's inventory with a join to the item table
    let inventory_items = UserInventory::find()
        .filter(
            crate::database::user_inventory::Column::UserId.eq(user_id.clone())
                .and(crate::database::user_inventory::Column::ServerId.eq(server_id.clone()))
        )
        .join(
            JoinType::InnerJoin,
            crate::database::user_inventory::Relation::Item.def()
        )
        .all(db)
        .await
        .context("Failed to get user inventory from database")?;

    debug!("Found {} inventory items", inventory_items.len());

    // Get the item details for each inventory item and filter for fish-related items
    let mut result = Vec::new();

    for inventory_item in inventory_items {
        let item = Item::find_by_id(inventory_item.item_id.clone())
            .one(db)
            .await
            .context(format!("Failed to get item details for item_id={}", inventory_item.item_id))?;

        if let Some(item) = item {
            // Include all fish-related items
            // This includes items with type "fish" and any other fish types
            // We'll consider an item to be fish-related if:
            // 1. It has type "fish" OR
            // 2. It has size and rarity attributes (which are typical for fish)
            if item.r#type == "fish" || 
               (inventory_item.size > 0 && inventory_item.rarity > 0) {
                result.push((inventory_item, item));
            }
        }
    }

    debug!("Found {} fish items", result.len());

    Ok(result)
}
