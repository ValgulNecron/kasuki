//! The `SeekCommand` struct and its implementation are used to handle the "seek" interaction
//! in a bot, allowing users to seek or jump to a specific point within the currently playing track.
//!
//! # Struct Fields
//! - `ctx: SerenityContext`
//!   - Provides the context of the interaction, including the bot's state.
//! - `command_interaction: CommandInteraction`
//!   - Represents the user interaction with the command.
//!
//! # Traits Implemented
//! ## `Command`
//! The `SeekCommand` struct implements the `Command` trait, enabling it to fulfill the behavior
//! required for handling seek operations in music playback. Below are the implemented methods:
//!
//! ### `get_ctx(&self) -> &SerenityContext`
//! Returns the bot context associated with the current command.
//!
//! ### `get_command_interaction(&self) -> &CommandInteraction`
//! Returns the interaction that triggered the command.
//!
//! ### `get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>>`
//! Handles the main logic for the seek command, including:
//! - Retrieving and verifying the guild ID to which the command belongs.
//! - Loading localized messages for responses based on the guild.
//! - Checking the availability of the Lavalink music client.
//! - Attempting to get the current music player for the guild.
//! - Seeking to the specified point in the currently playing track.
//! - Building response embeds with appropriate output messages (e.g., success or error).
//!
//! #### Behavior of `get_contents`
//! 1. The function defers the reply to the interaction for processing.
//! 2. Loads localized strings for the seek command using the guild ID.
//! 3. Fetches a reference to the Lavalink client for music playback management:
//!    - If Lavalink is disabled or uninitialized, returns an error message in an embed.
//! 4. Extracts the "time" option provided by the user (via subcommand input) to determine
//!    the seek position in seconds.
//! 5. Checks if a track is currently playing in the guild:
//!    - If a track is playing, seeks the track to the specified position.
//!    - If no track is playing, returns an appropriate error message as a response.
//! 6. Constructs an embed with the success or error result.
//! 7. Returns the generated embed content.
//!
//! #### Errors
//! The `get_contents` method produces an error if:
//! - Lavalink is disabled or unavailable.
//! - The guild has no active voice connection or player.
//! - The "time" option is missing, or seeking to the given position fails.
//!
//! ```
//! // Example usage:
//! let seek_command = SeekCommand {
//!     ctx: serenity_context,
//!     command_interaction,
//! };
//! seek_command.get_contents().await?;
//! ```
//!
//! # Dependencies
//! - Uses the Lavalink library to handle music playback.
//! - Depends on localized messages loaded via `load_localization_seek`.
//! - Relies on extracted subcommand options for the seek time.
//!
//! # Notes
//! - This command only functions within a guild context where Lavalink is properly configured.
//! - If no track is currently playing in the guild, the command will notify the user accordingly.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use crate::structure::message::music::seek::load_localization_seek;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::time::Duration;

/// The `SeekCommand` struct represents a command used to handle seeking functionality
/// in a Discord bot context utilizing the Serenity framework.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext` object representing the contextual data of the bot,
///           such as cache, HTTP, and other necessary components to operate commands.
///
/// * `command_interaction` - Represents the specific interaction object that triggered
///                           this command, encapsulating details like the user input,
///                           interaction ID, and other associated metadata.
///
/// # Usage
/// This struct is typically used to encapsulate the data required to execute a "seek"
/// command, which allows users to fast forward or rewind media playback during bot
/// interactions in a Discord server.
///
/// Example integration would involve the struct being processed by a handler function
/// to adjust playback state accordingly.
pub struct SeekCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SeekCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// This method provides access to the `ctx` field, which is of type `SerenityContext`.
	/// The `SerenityContext` typically contains various utilities and data required for
	/// interacting with the Discord API and handling bot state.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext`, allowing the caller to utilize its properties and methods.
	///
	/// # Example
	///
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use `ctx` to interact with Discord API or access bot state.
	/// ```
	///
	/// Note: Ensure the instance implementing this method contains a valid `ctx` field.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use `interaction` as needed
	/// ```
	///
	/// This method is useful for accessing the `CommandInteraction` object to inspect or
	/// interact with the underlying command details.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes audio playback information to create embedded response content.
	///
	/// This method performs the following steps:
	/// 1. Retrieves the current context and defers the command interaction for later response.
	/// 2. Extracts the Guild ID from the command interaction or assigns a default ID if not found.
	/// 3. Loads localized strings based on the Guild ID. These strings contain specific responses for user interaction.
	/// 4. Validates the bot's Lavalink client instance, which is required for audio playback. If unavailable, it returns an error or a response with a suitable message.
	/// 5. Extracts and parses the user's provided subcommand options, particularly the "time" value, which indicates the seek position in the audio.
	/// 6. Retrieves the currently playing track in the guild and, if a track is active, adjusts the playback to the specified seek position. Otherwise, a message indicating no track is playing is returned.
	///
	/// # Returns
	/// On success, it returns a `Vec` of `EmbedContent` objects, used to display appropriate feedback to the user (e.g., a success message or an error if no track is playing).
	///
	/// # Errors
	/// - Returns an error if:
	///   - The Lavalink client is disabled or unavailable.
	///   - There's an issue retrieving or modifying playback information.
	///   - Required command interaction details, such as Guild ID, are missing or invalid.
	/// - Custom error messages from the localization system may also be shown in the embedded content.
	///
	/// # Dependencies
	/// - The method relies on a Lavalink client (`lava_client`) for interacting with audio playback.
	/// - Guild-specific localized strings are loaded using the `load_localization_seek` function.
	/// - Global configurations and data are accessed through a `BotData` context.
	///
	/// # Parameters
	/// - `self`: Reference to the current instance, typically bound to a command interaction handler.
	///
	/// # Example
	/// This function is generally used within the bot's command-handling flow for seeking to a specific time in an audio track.
	///
	/// ```ignore
	/// // Example usage in a command handler
	/// let response = interaction_handler.get_contents().await;
	/// match response {
	///     Ok(embed_contents) => {
	///         // Send the embedded contents to the user
	///         for embed in embed_contents {
	///             interaction.follow_up(embed).await?;
	///         }
	///     }
	///     Err(err) => {
	///         // Log error or send a failure message
	///         error!("Failed to execute seek command: {}", err);
	///     }
	/// }
	/// ```
	async fn get_contents(&self) -> anyhow::Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match self.command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let seek_localised =
			load_localization_seek(guild_id_str, bot_data.config.db.clone()).await?;

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
			let embed_content =
				EmbedContent::new(seek_localised.title).description(seek_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

			return Ok(embed_contents);
		};

		let map = get_option_map_number_subcommand(command_interaction);

		let time = map.get(&String::from("time")).cloned().unwrap_or_default() as u64;

		let now_playing = player.get_player().await?.track;

		let mut embed_content = EmbedContent::new(seek_localised.title);

		if let Some(_) = now_playing {
			player.set_position(Duration::from_secs(time)).await?;
			embed_content =
				embed_content.description(seek_localised.success.replace("{0}", &time.to_string()));
		} else {
			embed_content = embed_content.description(seek_localised.nothing_playing);
		}

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}
