//! Module implementing the `CreditCommand` structure and its functionality.
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::bot::credit::load_localization_credit;
use serenity::all::{CommandInteraction, Context as SerenityContext};
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

		// Load the localized credit strings
		debug!("Loading credit localization for guild: {}", guild_id);
		let credit_localised = load_localization_credit(guild_id, db_connection).await?;
		debug!("Credit localization loaded successfully");

		// Construct a description by concatenating the descriptions of all credits
		debug!("Constructing credit description");
		let mut desc: String = "".to_string();

		for x in credit_localised.credits {
			desc += x.desc.as_str();
			debug!("Added credit description to the combined text");
		}

		debug!("Creating embed content");
		let title = credit_localised.title.clone();
		let embed_content = EmbedContent::new(credit_localised.title).description(desc);
		debug!("Embed content created with title: {}", title);

		debug!("Creating final embed contents with CommandType::First");
		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		info!("Credit command processed successfully");
		Ok(embed_contents)
	}
);
