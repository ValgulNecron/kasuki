//! ClearCommand is a struct that represents the clear command in a bot application.
//!
//! The command is triggered by users to clear the music queue in a voice channel. It processes
//! the command interaction, interacts with Lavalink for music management, and responds
//! to the user with appropriate messages.
//!
//! # Fields
//! - `ctx` - The Serenity context, containing information about the bot's connection and state.
//! - `command_interaction` - The interaction representing the command invocation.
//!
//! ## Implementations
//!
//! ### `impl Command for ClearCommand`
//!
//! Implements the `Command` trait, which defines the behavior for commands in the bot.
//!
//! #### Methods
//!
//! ##### `get_ctx(&self) -> &SerenityContext`
//! Returns a reference to the Serenity context associated with the command.
//!
//! ##### `get_command_interaction(&self) -> &CommandInteraction`
//! Returns a reference to the command interaction that triggered this command.
//!
//! ##### `async fn get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>>`
//! Handles the execution of the clear command and constructs a response for the user.
//!
//! - Fetches the bot's data store, command interaction, and other necessary resources.
//! - Ensures that the command is executed in a guild (server) by retrieving the `guild_id`.
//! - Loads localized clear command messages based on the guild's data.
//! - Checks for an active Lavalink client, which is required for music-related operations.
//! - If no active music player is found for the guild, returns an error response to the user.
//! - If an active player exists, clears the music queue and sends a success message.
//!
//! #### Behavior & Steps:
//! 1. Retrieves the Lavalink client and checks if it is operational.
//! 2. Ensures the command is executed in a valid voice channel context by checking the player context.
//! 3. Clears the player's music queue.
//! 4. Constructs and returns a success or error message based on the operation's result.
//!
//! #### Possible Errors
//! - Returns an error if the command is triggered outside a guild.
//! - Returns an error if the bot's Lavalink integration is unavailable or disabled.
//! - Returns an error if there is no active player in the voice channel or if queue clearing fails.
//!
//! # Dependencies
//! - `crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType}`:
//!   Traits and types for handling commands and constructing embed responses.
//! - `crate::event_handler::BotData`: Bot-specific data storage for configuration and Lavalink.
//! - `serenity::all::{CommandInteraction, Context as SerenityContext}`: Serenity-related types
//!   for handling Discord interactions.
//!
//! # Example
//! ```rust
//! let clear_command = ClearCommand {
//!     ctx,
//!     command_interaction
//! };
//!
//! clear_command.get_contents().await;
//! ```
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};

#[slash_command(
	name = "clear", desc = "Clear the current queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn clear_command(self_: ClearCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();

	let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized strings
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

	let lava_client = bot_data.lavalink.clone();
	let lava_client = lava_client.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();

	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_clear-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_clear-error_no_voice"));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		return Ok(embed_contents);
	};

	player.get_queue().clear()?;

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_clear-title")).description(USABLE_LOCALES.lookup(&lang_id, "music_clear-success"));

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
	Ok(embed_contents)
}
