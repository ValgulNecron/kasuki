use crate::command::guess_kind::guess_command_kind;
use crate::command::registry::{get_message_registry, get_slash_registry, get_user_registry};
use crate::event_handler::BotData;
use anyhow::{Context as AnyhowContext, Result};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};

#[instrument(name = "dispatch_command", skip(ctx, command_interaction), fields(
	user_id = ?command_interaction.user.id,
	guild_id = ?command_interaction.guild_id,
))]
pub async fn dispatch_command(
	ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> Result<()> {
	info!(
		"Dispatching command from user: {} (ID: {})",
		command_interaction.user.name, command_interaction.user.id
	);

	let bot_data = ctx.data::<BotData>().clone();
	let (kind, name) = guess_command_kind(command_interaction);
	let full_command_name = format!("{} {}", kind, name);

	debug!(
		"Command details: type={}, name={}, full_name={}",
		kind, name, full_command_name
	);

	let start_time = Instant::now();
	info!("Executing command: {}", full_command_name);

	let handler = get_slash_registry().get(name.as_str()).ok_or_else(|| {
		error!("Unknown command requested: {}", full_command_name);
		anyhow::anyhow!("Command not found: {}", full_command_name)
	})?;

	handler
		.run(ctx, command_interaction, &full_command_name)
		.await
		.context(format!("Error executing command: {}", full_command_name))?;

	let execution_time = start_time.elapsed();
	debug!(
		"Command {} execution took {:?}",
		full_command_name, execution_time
	);

	if execution_time.as_millis() > 1000 {
		warn!(
			"Command {} took over 1 second to execute: {:?}",
			full_command_name, execution_time
		);
	}

	bot_data
		.increment_command_use_per_command(
			full_command_name.clone(),
			command_interaction.user.id.to_string(),
			command_interaction.user.name.to_string(),
		)
		.await;

	info!("Command {} executed successfully", full_command_name);
	Ok(())
}

pub async fn dispatch_user_command(
	ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> Result<()> {
	let name = command_interaction.data.name.as_str();
	let handler = get_user_registry()
		.get(name)
		.ok_or_else(|| anyhow::anyhow!("Unknown user command: {}", name))?;
	handler.run(ctx, command_interaction, name).await
}

pub async fn dispatch_message_command(
	ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> Result<()> {
	let name = command_interaction.data.name.as_str();
	let handler = get_message_registry()
		.get(name)
		.ok_or_else(|| anyhow::anyhow!("Unknown message command: {}", name))?;
	handler.run(ctx, command_interaction, name).await
}
