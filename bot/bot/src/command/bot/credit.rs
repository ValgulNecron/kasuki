//! Module implementing the `CreditCommand` structure and its functionality.
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use tracing::{debug, info};

#[slash_command(
	name = "credit", desc = "Get the credit of the app.",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn credit_command(self_: CreditCommand) -> Result<EmbedsContents<'_>> {
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

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	info!("Credit command processed successfully");
	Ok(embed_contents)
}
