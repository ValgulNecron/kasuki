use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::{debug, info, trace, warn};

#[slash_command(
	name = "ping", desc = "Get the ping of the bot (and the shard id).",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn ping_command(self_: PingCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing ping command");
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	debug!("Retrieving bot data and configuration");

	// Get the language identifier for the guild
	let lang_id = cx.lang_id().await;

	debug!("Retrieving shard manager from bot data");
	let shard_runner_info = cx.ctx.runner_info.read();

	// Retrieve the shard ID from the context
	let shard_id = cx.ctx.shard_id;
	debug!("Current shard ID: {}", shard_id);

	// Retrieve the shard runner info from the shard manager
	debug!("Retrieving shard runner info for shard {}", shard_id);
	let (latency, stage) = {
		// Format the latency as a string
		let latency = match shard_runner_info.latency {
			Some(latency) => {
				let formatted = format!("{:.2}ms", latency.as_millis());
				debug!("Shard {} latency: {}", shard_id, formatted);
				formatted
			},
			None => {
				warn!("Latency information not available for shard {}", shard_id);
				"?,??ms".to_string()
			},
		};

		// Retrieve the stage of the shard runner
		let stage = shard_runner_info.stage.to_string();
		debug!("Shard {} connection stage: {}", shard_id, stage);
		drop(shard_runner_info);
		(latency, stage)
	};

	debug!("Creating embed content with ping information");
	let mut args = HashMap::new();
	args.insert(
		Cow::Borrowed("shard"),
		FluentValue::from(shard_id.to_string()),
	);
	args.insert(Cow::Borrowed("latency"), FluentValue::from(latency));
	args.insert(Cow::Borrowed("status"), FluentValue::from(stage));

	let title = USABLE_LOCALES.lookup(&lang_id, "bot_ping-title");
	let description = USABLE_LOCALES.lookup_with_args(&lang_id, "bot_ping-desc", &args);

	trace!("Formatted ping description: {}", description);

	let embed_content = EmbedContent::new(title.clone()).description(description);
	debug!("Embed content created with title: {}", title);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	info!("Ping command processed successfully");
	Ok(embed_contents)
}
