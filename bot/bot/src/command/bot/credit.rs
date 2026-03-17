//! Module implementing the `CreditCommand` structure and its functionality.
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use tracing::{debug, info};

#[slash_command(
	name = "credit", desc = "Get the credit of the app.",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn credit_command(self_: CreditCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing credit command");
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	debug!("Retrieving bot data and configuration");

	// Get the language identifier for the guild
	debug!("Loading localization for guild: {}", cx.guild_id);
	let lang_id = cx.lang_id().await;
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
