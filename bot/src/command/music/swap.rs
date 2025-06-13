//! The `SwapCommand` struct and its implementation define the functionality to swap two tracks
//! in a music player's queue in response to a command interaction.
//!
//! # Fields
//! - `ctx`: The context from Serenity, used to interact with the Discord bot.
//! - `command_interaction`: An interaction object representing the executed command.
//!
//! # Implementation
//! The `SwapCommand` struct implements the `Command` trait, which provides the following functionality:
//!
//! ## Methods
//!
//! ### `get_ctx`
//! Returns a reference to the Serenity context.
//!
//! ```rust
//! fn get_ctx(&self) -> &SerenityContext
//! ```
//!
//! ### `get_command_interaction`
//! Returns a reference to the command interaction object.
//!
//! ```rust
//! fn get_command_interaction(&self) -> &CommandInteraction
//! ```
//!
//! ### `get_contents`
//! Handles the core logic for processing the "swap" command. It performs the following:
//! - Retrieves the necessary bot data from the context.
//! - Defers the interaction response to indicate processing time.
//! - Loads localization strings for the swap command based on the guild ID.
//! - Validates if the Lavalink music player client is enabled.
//! - Checks if the indices (`index1`, `index2`) provided by the user are valid for the queue.
//! - Swaps the tracks at the specified indices in the player's music queue.
//! - Builds an embed response to indicate success or errors (e.g., invalid indices, Lavalink not running).
//!
//! #### Parameters
//! None directly (uses `self`).
//!
//! #### Returns
//! An `anyhow::Result` containing `EmbedsContents` on success, or an error if something went wrong.
//!
//! #### Example Workflow
//! 1. Determines if the command interaction has a valid guild ID.
//! 2. Loads localized strings for success or error messages.
//! 3. Retrieves the Lavalink player's queue for the current guild.
//! 4. Validates and processes the `index1` and `index2` options from the command interaction.
//! 5. Swaps the tracks in the queue and constructs an embed message with the result.
//!
//! #### Important Details
//! - If no valid guild ID is found, an error is returned.
//! - If the indices are greater than the queue length or the same, appropriate error messages are shown using embed content.
//! - If Lavalink is disabled or unavailable, the command fails.
//!
//! ```rust
//! async fn get_contents(&self) -> anyhow::Result<EmbedsContents>
//! ```
//!
//! # Dependencies
//! This implementation uses several modules and crates:
//! - `serenity::all`: Handles Discord API interactions.
//! - Local modules such as:
//!   - `command`: Defines command-related structs and enums (e.g., `Command`, `EmbedContent`).
//!   - `event_handler`: Provides bot data configurations.
//!   - `helper`: Includes utility functions (e.g., retrieving command options).
//! - `anyhow`: For error handling.
//! - `lavalink_rs`: Manages interactions with the Lavalink music streaming server.
//!
//! # Errors
//! The `get_contents` method may return errors in the following scenarios:
//! - Missing guild ID (`no guild id`).
//! - Lavalink is not enabled or available.
//! - Invalid or missing indices to swap in the queue.
//!
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use crate::structure::message::music::swap::load_localization_swap;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// Represents a command structure used to handle a "swap" operation within a Discord bot.
///
/// This structure encapsulates the context of the interaction and the associated command interaction
/// for easier manipulation and access.
///
/// # Fields
///
/// * `ctx` - The context of the bot, providing access to the Discord API, bot state, and necessary utilities
///   to process the command. This is an instance of `SerenityContext`.
///
/// * `command_interaction` - Represents the details of the user's input or request when invoking the command.
///   This includes any command arguments or metadata about the interaction,
///   provided as a `CommandInteraction`.
///
/// # Usage
///
/// The `SwapCommand` structure is designed to abstract and streamline the handling of a specific command within
/// the bot. It provides direct access to both the context and the interaction, which are typically essential
/// components for most command-processing logic.
pub struct SwapCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SwapCommand {
	/// Retrieves a reference to the `SerenityContext`.
	///
	/// This function provides access to the Discord bot's context,
	/// allowing the caller to utilize the context for various operations, such as interacting
	/// with Discord's API, managing bot state, or listening to events.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext`.
	///
	/// # Example
	///
	/// ```rust
	/// let context = my_struct.get_ctx();
	/// // Use `context` to interact with Discord API or handle various tasks
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance.
	///
	/// # Examples
	/// ```rust
	/// let interaction = object.get_command_interaction();
	/// // Use the `interaction` reference as needed
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves the embed contents for swapping tracks in a music playback queue.
	///
	/// # Arguments
	/// - `self`: The context for the command interaction, which contains necessary data to process the command.
	///
	/// # Returns
	/// - `anyhow::Result<EmbedsContents>`: An `EmbedsContents` object that contains the response to the user
	///   based on the processing of the swap action.
	///
	/// # Workflow
	/// 1. Fetch application context and bot data that's used throughout the function.
	/// 2. Defer the command interaction to prevent timeouts while processing.
	/// 3. Retrieve the Guild ID associated with the command interaction.
	/// 4. Load localized content for the swap operation based on database configuration.
	/// 5. Validate if Lavalink (music client) is enabled and fetch its instance.
	///    If Lavalink is disabled, return an error response embedded with a localized message.
	/// 6. Retrieve the player context for the current guild. If no player context exists,
	///    send a message indicating that the user is not in a voice channel.
	/// 7. Extract the indices (`index1` and `index2`) from the command interaction's input options for track swapping.
	///    Defaults to `0` if indices are missing.
	/// 8. Validate the extracted indices:
	///    - If either index exceeds the queue length or indices are identical, return respective error messages.
	/// 9. Perform a swap operation between the two tracks in the queue using their indices:
	///    - Fetch the tracks at the given indices.
	///    - Swap the tracks in the playback queue.
	///    - Return a success message.
	/// 10. Compile the appropriate embed content (error or success) and return it to the user.
	///
	/// # Errors
	/// - Returns an error if:
	///   - The Guild ID is missing in the command interaction.
	///   - Lavalink client is disabled or unavailable.
	///   - Any unexpected issues occur while fetching data, interacting with the Lavalink client,
	///     or manipulating the track queue.
	///
	/// # Example
	/// ```ignore
	/// // Assuming this function is invoked during a music bot operation when a user
	/// // issues a "swap" command to switch the positions of two tracks in the queue.
	/// let embed_contents = command.get_contents().await?;
	/// for embed in embed_contents {
	///     send_followup_command(embed).await?;
	/// }
	/// ```
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();

		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match self.command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let swap_localised =
			load_localization_swap(guild_id_str, bot_data.config.db.clone()).await?;

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
				EmbedContent::new(swap_localised.title).description(swap_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

			return Ok(embed_contents);
		};

		let map = get_option_map_number_subcommand(command_interaction);

		let index1 = map
			.get(&String::from("index1"))
			.cloned()
			.unwrap_or_default() as usize;

		let index2 = map
			.get(&String::from("index2"))
			.cloned()
			.unwrap_or_default() as usize;

		let mut embed_content = EmbedContent::new(swap_localised.title);

		let queue = player.get_queue();
		let queue_len = queue.get_count().await?;

		if index1 > queue_len || index2 > queue_len {
			embed_content = embed_content.description(
				swap_localised
					.error_max_index
					.replace("{0}", &queue_len.to_string()),
			);
		} else if index1 == index2 {
			embed_content = embed_content.description(swap_localised.error_same_index);
		} else {
			let track1 = queue.get_track(index1 - 1).await?.unwrap();
			let track2 = queue.get_track(index1 - 2).await?.unwrap();

			queue.swap(index1 - 1, track2)?;
			queue.swap(index2 - 1, track1)?;

			embed_content = embed_content.description(swap_localised.success);
		}

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}
