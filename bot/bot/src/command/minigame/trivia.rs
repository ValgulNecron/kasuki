use crate::command::command::{Command, CommandRun};
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::{Context as AnyhowContext, Result};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::item::{Entity as Item, Model as ItemModel};
use shared::database::user_inventory::{Entity as UserInventory, Model as UserInventoryModel};
use std::collections::HashMap;
use tracing::debug;

#[derive(Clone)]
pub struct TriviaCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for TriviaCommand,
	get_contents = |self_: InventoryCommand| async move {
		let cx = CommandContext::new(
			self_.get_ctx().clone(),
			self_.get_command_interaction().clone(),
		);

	}
);