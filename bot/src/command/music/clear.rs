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
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::structure::message::music::clear::load_localization_clear;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A structure representing a `ClearCommand` used to handle the `clear` command
/// in a Discord bot using the Serenity library.
///
/// This structure holds the context of the bot and the interaction data associated
/// with the `clear` command triggered by a user in a Discord server.
///
/// # Fields
///
/// * `ctx` - The context of the Serenity framework, providing access to bot data such as cache,
///   HTTP client, and other bot-related functions. This allows the command to interact with
///   Discord's API and manage server data.
///
/// * `command_interaction` - The interaction data associated with the command. This contains information
///   such as user input, the channel where the command was invoked, and other metadata about the command
///   interaction.
///
/// # Example
///
/// ```rust
/// let clear_command = ClearCommand {
///     ctx: serenity_ctx,
///     command_interaction: command_interaction_data,
/// };
/// ```
///
/// The `ClearCommand` struct is expected to be used to process and execute the logic for clearing messages
/// or performing other related administrative actions in a Discord server.
pub struct ClearCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ClearCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// This method returns the context (`SerenityContext`) that can be used to interact with Discord's API,
	/// manage the current bot state, or perform other context-specific actions.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` held by the current instance.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` to interact with Discord API or manage bot state
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with this instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` field.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use the `interaction` as needed
	/// ```
	///
	/// # Notes
	/// This method provides read-only access to the `CommandInteraction`
	/// field of the structure it is called on.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the contents resulting from a Lavalink queue clear operation.
	///
	/// # Description
	/// This method processes the command interaction and clears the player's playback queue
	/// through Lavalink. It handles various scenarios such as missing guild context or an inactive
	/// Lavalink service, and provides localized responses accordingly.
	///
	/// # Returns
	/// - Returns a `Result` containing a vector of `EmbedContent` upon success, representing localized success
	///   or error messages.
	/// - Returns an `anyhow::Error` if the operation fails, such as due to missing data or configuration errors.
	///
	/// # Errors
	/// This function may return the following errors:
	/// - `anyhow::Error` if the guild ID is unavailable in the command interaction.
	/// - `anyhow::Error` if Lavalink is unavailable or disabled.
	/// - `anyhow::Error` if any unexpected issue occurs during processing.
	///
	/// # Example
	/// ```rust
	/// // Assuming `self` implements the necessary trait context
	/// let contents = self.get_contents().await;
	/// match contents {
	///     Ok(embeds) => {
	///         for embed in embeds {
	///             // Process or send the embedded content
	///             println!("Embed title: {}", embed.title());
	///         }
	///     }
	///     Err(err) => eprintln!("Failed to retrieve contents: {}", err),
	/// }
	/// ```
	///
	/// # Notes
	/// - This function defers the interaction before performing resource-intensive operations.
	/// - Localized strings are loaded based on the guild ID associated with the command interaction.
	/// - The player's Lavalink queue is cleared at the end of this procedure if the operation is successful.
	///
	/// # Dependencies
	/// - `anyhow` for error handling.
	/// - `EmbedContent` for formatting interaction responses.
	/// - `lavalink_rs` for managing audio playback and player state in the Lavalink client.
	///
	/// # Steps Overview
	/// 1. Retrieve command interaction and guild ID.
	/// 2. Load localized strings for the `clear` command.
	/// 3. Check for the availability of a Lavalink client instance.
	/// 4. Attempt to retrieve the player for the given guild.
	/// 5. On success, clear the player's queue and respond with a localized success message.
	/// 6. Handle errors and provide appropriate localized failure messages.
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let clear_localised =
			load_localization_clear(guild_id_str, bot_data.config.db.clone()).await?;

		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		if lava_client.is_none() {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		}
		let lava_client = lava_client.unwrap();

		let Some(player) =
			lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		else {
			let embed_content = EmbedContent::new(clear_localised.title)
				.description(clear_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

			return Ok(embed_contents);
		};

		player.get_queue().clear()?;

		let embed_content =
			EmbedContent::new(clear_localised.title).description(clear_localised.success);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
		Ok(embed_contents)
	}
}
