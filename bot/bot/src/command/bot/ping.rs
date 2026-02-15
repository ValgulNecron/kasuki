use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::{debug, error, info, trace, warn};

#[slash_command(
	name = "ping", desc = "Get the ping of the bot (and the shard id).",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn ping_command(self_: PingCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing ping command");
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let _config = &bot_data.config;

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
	let lang_id = get_language_identifier(guild_id, db_connection).await;

	debug!("Retrieving shard manager from bot data");
	let guard = ctx.data::<BotData>().shard_manager.clone();
	let guard = guard.read().await;
	let manager = guard.clone();
	drop(guard);
	let shard_manager = match manager {
		Some(shard_manager) => {
			debug!("Successfully retrieved shard manager");
			shard_manager.clone()
		},
		None => {
			error!("Failed to get shard manager from bot data");
			return Err(anyhow!("failed to get the shard manager"));
		},
	};

	// Retrieve the shard ID from the context
	let shard_id = ctx.shard_id;
	debug!("Current shard ID: {}", shard_id);

	// Retrieve the shard runner info from the shard manager
	debug!("Retrieving shard runner info for shard {}", shard_id);
	let (latency, stage) = {
		let shard_runner_info = match shard_manager.get(&shard_id) {
			Some(info) => {
				debug!("Found shard runner info for shard {}", shard_id);
				info
			},
			None => {
				error!("Failed to get shard info for shard {}", shard_id);
				return Err(anyhow!("failed to get the shard info"));
			},
		};

		// Format the latency as a string
		let (info, _) = shard_runner_info.value();
		let latency = match info.latency {
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
		let stage = info.stage.to_string();
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

	debug!("Creating final embed contents with CommandType::First");
	let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

	info!("Ping command processed successfully");
	Ok(embed_contents)
}
