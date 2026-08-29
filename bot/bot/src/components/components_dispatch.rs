use anyhow::Result;
use sea_orm::DatabaseConnection;
use serenity::all::{ComponentInteraction, Context as SerenityContext};
use std::sync::Arc;
use tracing::trace;

use crate::components::handler::ComponentHandler;

pub async fn components_dispatching(
	ctx: &SerenityContext, component_interaction: &ComponentInteraction,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let custom_id = component_interaction.data.custom_id.as_str();

	// Iterate all component handlers registered via inventory::submit! across the crate.
	// Each handler declares a prefix (e.g. "anime_") and whether to use prefix matching.
	// First match wins — order depends on linker, so prefixes must be non-overlapping.
	for handler in inventory::iter::<&'static dyn ComponentHandler> {
		if handler.match_prefix() && custom_id.starts_with(handler.prefix()) {
			handler
				.handle(ctx, component_interaction, db_connection)
				.await?;
			return Ok(());
		}
	}

	// No handler matched — silently ignore; Discord may send stale component interactions
	trace!("does not exist.");
	Ok(())
}
