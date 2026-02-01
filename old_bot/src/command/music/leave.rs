//! This module defines and implements the `LeaveCommand` structure, which is a concrete
//! implementation of the `Command` trait. The purpose of the `LeaveCommand` is to handle
//! the "leave" command functionality, primarily for managing the bot's disconnection
//! from voice channels in a guild context.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::music::leave::load_localization_leave;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing a command used to handle a "leave" interaction in a Discord bot.
///
/// The `LeaveCommand` struct contains context and interaction data necessary for executing the command
/// when invoked by a user.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext` provides the bot's context, including connection details,
///           interaction with Discord API, and shared state.
///
/// * `command_interaction` - The `CommandInteraction` holds the details of the user's invoked command.
///                           This includes metadata (e.g., command arguments, associated user, etc.).
///
/// # Purpose
///
/// This struct is specifically designed to encapsulate the components required for handling
/// "leave" command logic, which is typically used to make the bot leave a channel, server, or
/// audio session, depending on the bot functionality.
///
/// # Example Usage
///
/// This struct would typically be initialized and passed to a handling function that executes
/// the "leave" logic:
///
/// ```rust
/// let leave_command = LeaveCommand {
///     ctx,
///     command_interaction,
/// };
/// leave_command.execute().await;
/// ```
#[derive(Clone)]
pub struct LeaveCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for LeaveCommand,
	get_contents = |self_: LeaveCommand| async move {
		self_.defer().await?;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let db_connection = bot_data.db_connection.clone();

		// Load the localized strings
		let leave_localised = load_localization_leave(guild_id_str, db_connection).await?;

		let manager = bot_data.manager.clone();
		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		if lava_client.is_none() {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		}
		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

		let lava_client = lava_client.unwrap();

		lava_client
			.delete_player(lavalink_rs::model::GuildId::from(guild_id.get()))
			.await?;

		if manager.get(guild_id).is_some() {
			manager.remove(guild_id).await?;
		}

		let embed_content =
			EmbedContent::new(leave_localised.title).description(leave_localised.success);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);
