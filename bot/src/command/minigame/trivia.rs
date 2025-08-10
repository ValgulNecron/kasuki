use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::item::{Entity as Item, Model as ItemModel};
use crate::database::user_inventory::{Entity as UserInventory, Model as UserInventoryModel};
use crate::event_handler::BotData;
use crate::impl_command;
use anyhow::{Context as AnyhowContext, Result};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
};
use serenity::all::{CommandInteraction, Context as SerenityContext};
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
        self_.defer().await;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

	}
);