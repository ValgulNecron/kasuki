//! The `PauseCommand` struct represents a command to pause the currently playing
//! track in a voice channel. This command is invoked through a user interaction
//! (slash command) in a Discord server.
//!
//! # Fields
//! * `ctx` - An instance of `SerenityContext` providing access to Discord's API.
//! * `command_interaction` - The CommandInteraction object containing information about the interaction.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::music::pause::load_localization_pause;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// `PauseCommand` is a structure that encapsulates data required to handle a "pause" command interaction
/// in a Discord bot using the Serenity library.
///
/// # Fields
/// - `ctx` (SerenityContext): The context of the bot, which gives access to various utilities and information
///   about the current bot state, guild, or shard operations.
/// - `command_interaction` (CommandInteraction): The interaction representing the command issued by a user,
///   which contains details such as the user, channel, and any associated input data.
///
/// This structure allows easier handling and encapsulation of pause command functionality.
#[derive(Clone)]
pub struct PauseCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for PauseCommand,
	get_contents = |self_: PauseCommand| async move {
		self_.defer().await?;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match self_.command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let db_connection = bot_data.db_connection.clone();

		// Load the localized strings
		let pause_localised = load_localization_pause(guild_id_str, db_connection).await?;

		let command_interaction = self_.get_command_interaction();

		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		if lava_client.is_none() {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		}
		let lava_client = lava_client.unwrap();

		let Some(player) =
			lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		else {
			let embed_content = EmbedContent::new(pause_localised.title)
				.description(pause_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
			return Ok(embed_contents);
		};
		player.set_pause(true).await?;

		let embed_content =
			EmbedContent::new(pause_localised.title).description(pause_localised.success);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
		Ok(embed_contents)
	}
);
