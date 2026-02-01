//! Module implementing the `CreditCommand` structure and its functionality.
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use tracing::{debug, info};

#[derive(Clone)]
pub struct CreditCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for CreditCommand,
	get_contents = |self_: CreditCommand| async move {
		info!("Processing credit command");
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let _config = bot_data.config.clone();

		debug!("Retrieving bot data and configuration");

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => {
				debug!("Command executed in guild: {}", id);
				id.to_string()
			},
			None => {
				debug!("Command executed in DM");
				String::from("0")
			},
		};
		let db_connection = bot_data.db_connection.clone();

		// Get the language identifier for the guild
		debug!("Loading localization for guild: {}", guild_id);
		let lang_id = get_language_identifier(guild_id, db_connection).await;
		debug!("Localization loaded successfully");

		debug!("Creating embed content");
		let title = USABLE_LOCALES.lookup(&lang_id, "bot_credit-title");
		let desc = USABLE_LOCALES.lookup(&lang_id, "bot_credit-desc");
		let embed_content = EmbedContent::new(title.clone()).description(desc);
		debug!("Embed content created with title: {}", title);

		debug!("Creating final embed contents with CommandType::First");
		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		info!("Credit command processed successfully");
		Ok(embed_contents)
	}
);
