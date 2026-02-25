use anyhow::Result;
use sea_orm::DatabaseConnection;
use serenity::all::{ComponentInteraction, Context as SerenityContext};
use std::sync::Arc;
use tracing::trace;

use crate::components::handler::ComponentHandler;

pub async fn components_dispatching(
	ctx: SerenityContext, component_interaction: ComponentInteraction,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let custom_id = component_interaction.data.custom_id.as_str();

	for handler in inventory::iter::<&'static dyn ComponentHandler> {
		if handler.match_prefix() && custom_id.starts_with(handler.prefix()) {
			handler
				.handle(&ctx, &component_interaction, db_connection)
				.await?;
			return Ok(());
		}
	}

	trace!("does not exist.");
	Ok(())
}
