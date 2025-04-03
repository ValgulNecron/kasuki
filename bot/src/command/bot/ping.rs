use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::bot::ping::load_localization_ping;
use anyhow::{Result, anyhow};
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use std::sync::Arc;
pub struct PingCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for PingCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for PingCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	// Retrieve the guild ID from the command interaction
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized ping strings
	let ping_localised = load_localization_ping(guild_id, config.db.clone()).await?;

	let guard = ctx.data::<BotData>().shard_manager.clone();
	let guard = guard.read().await;
	let manager = guard.clone();
	drop(guard);
	let shard_manager = match manager {
		Some(shard_manager) => shard_manager.clone(),
		None => {
			return Err(anyhow!("failed to get the shard manager"));
		},
	};

	// Retrieve the shard ID from the context
	let shard_id = ctx.shard_id;
	// Retrieve the shard runner info from the shard manager
	let (latency, stage) = {
		let shard_runner_info_lock = shard_manager.clone();
		let shard_runner_info = shard_runner_info_lock
			.get(&shard_id)
			.ok_or(anyhow!("failed to get the shard info"))?;
		// Format the latency as a string
		let shard_runner_info = shard_runner_info;
		let shard_runner_info = match shard_runner_info.lock() {
			Ok(shard_runner_info) => shard_runner_info,
			Err(_) => {
				return Err(anyhow!("failed to get the shard runner info"));
			},
		};
		let latency = match shard_runner_info.latency {
			Some(latency) => format!("{:.2}ms", latency.as_millis()),
			None => "?,??ms".to_string(),
		};

		// Retrieve the stage of the shard runner
		let stage = shard_runner_info.stage.to_string();
		drop(shard_runner_info);
		(latency, stage)
	};

	// Construct the embed for the response
	let builder_embed = get_default_embed(None)
		.description(
			ping_localised
				.desc
				.replace("$shard$", shard_id.to_string().as_str())
				.replace("$latency$", latency.as_str())
				.replace("$status$", &stage),
		)
		.title(&ping_localised.title);

	// Construct the message for the response
	let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

	// Construct the response
	let builder = CreateInteractionResponse::Message(builder_message);

	// Send the response to the command interaction
	command_interaction
		.create_response(&ctx.http, builder)
		.await?;

	Ok(())
}
