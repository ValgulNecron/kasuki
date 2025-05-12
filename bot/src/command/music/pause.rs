//! The `PauseCommand` struct represents a command to pause the currently playing
//! track in a voice channel. This command is invoked through a user interaction
//! (slash command) in a Discord server.
//!
//! # Fields
//! * `ctx` - An instance of `SerenityContext` providing access to Discord's API.
//! * `command_interaction` - The CommandInteraction object containing information about the interaction.
use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
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
pub struct PauseCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for PauseCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) which provides access
	/// to the internal state of the Discord bot, such as configurations, cache, and other
	/// utility functions.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the context for further operations
	/// ```
	///
	/// This function is useful when you need to interact with the bot's context, such as
	/// sending messages or accessing other Discord-related resources.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object stored in the instance.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	///
	/// Use this method to access the stored `CommandInteraction` for reading or processing without taking ownership.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the contents of an embed for a pause command.
	///
	/// This function handles contextual operations and interactions for managing a music player in a
	/// bot. It performs the following tasks:
	/// - Retrieves the necessary context and bot-specific data.
	/// - Defers the command interaction to inform the user that processing is ongoing.
	/// - Fetches the guild ID associated with the command interaction and loads localization strings 
	///   that correspond to the identified guild.
	/// - Ensures that the Lavalink music player client is active. If disabled or unavailable, an error 
	///   is returned.
	/// - Attempts to retrieve the player context for the guild from the Lavalink client.
	/// - Sends an appropriate embed message if there is an error (e.g., user not connected to a voice 
	///   channel) or if the command cannot be performed.
	/// - Pauses the currently playing track and returns a success embed message upon successful 
	///   execution.
	///
	/// # Returns
	/// A `Result` containing a vector of [`EmbedContent`] with an embed response for the command, or
	/// an error if any operation fails.
	///
	/// # Errors
	/// The function can return errors in the following scenarios:
	/// - Guild ID is not present in the command interaction.
	/// - Localization strings fail to load for the guild.
	/// - The Lavalink client is disabled or unavailable.
	/// - User is not connected to a voice channel or there is no player found for the guild.
	/// - Any Lavalink operation (e.g., pausing the player) fails.
	///
	/// # Usage
	/// This function is typically called when handling a bot command to pause the music currently 
	/// playing in the user's voice channel.
	///
	/// # Dependencies
	/// - This function relies on the `anyhow` crate for error handling.
	/// - Expects a valid Lavalink client and localized configuration strings to be available in the 
	///   bot's context.
	///
	/// # Example
	/// ```rust
	/// let result = some_instance.get_contents().await;
	/// match result {
	///     Ok(contents) => {
	///         // Send response embed to the user
	///     }
	///     Err(err) => {
	///         // Handle and log error
	///     }
	/// }
	/// ```
	///
	/// # Related Structures
	/// - [`EmbedContent`]: Represents the content of embed messages sent to the user.
	/// - [`EmbedType`]: Type of the embed message (e.g., follow-up, error).
	///
	async fn get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match self.command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let pause_localised =
			load_localization_pause(guild_id_str, bot_data.config.db.clone()).await?;

		let command_interaction = self.get_command_interaction();

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
				.description(pause_localised.error_no_voice)
				.command_type(EmbedType::Followup);
			return Ok(vec![embed_content]);
		};
		player.set_pause(true).await?;

		let embed_content = EmbedContent::new(pause_localised.title)
			.description(pause_localised.success)
			.command_type(EmbedType::Followup);
		
		Ok(vec![embed_content])
	}
}
