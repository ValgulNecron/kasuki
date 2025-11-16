use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::bot::ping::load_localization_ping;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::{debug, error, info, trace, warn};

#[derive(Clone)]
pub struct PingCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for PingCommand,
	get_contents = |self_: PingCommand| async move {
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

		// Load the localized ping strings
		debug!("Loading ping localization for guild: {}", guild_id);
		let ping_localised = load_localization_ping(guild_id, db_connection)
			.await
			.map_err(|e| {
				error!("Failed to load ping localization: {}", e);
				e
			})?;
		debug!("Ping localization loaded successfully");

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
		let description = ping_localised
			.desc
			.replace("$shard$", shard_id.to_string().as_str())
			.replace("$latency$", latency.as_str())
			.replace("$status$", &stage);

		trace!("Formatted ping description: {}", description);

		let embed_content =
			EmbedContent::new(ping_localised.title.clone()).description(description);
		debug!("Embed content created with title: {}", ping_localised.title);

		debug!("Creating final embed contents with CommandType::First");
		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		info!("Ping command processed successfully");
		Ok(embed_contents)
	}
);
