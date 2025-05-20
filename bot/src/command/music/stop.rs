//! The `StopCommand` struct represents a command that stops music playback in a voice channel.
//! It implements the `Command` trait representing a bot command interface.
//!
//! # Fields
//! - `ctx`: The context of the bot, used to access shared data, including bot state and framework.
//! - `command_interaction`: Represents the slash command interaction that triggered this command.
//!
//! # Trait Implementations
//!
//! ## `Command`
//!
//! ### Methods
//!
//! - `get_ctx`
//!   Retrieves the bot context (`SerenityContext`) associated with this command execution.
//!   This context provides access to bot-related data needed for command logic.
//!
//! - `get_command_interaction`
//!   Retrieves the slash command interaction (`CommandInteraction`) that triggered this command.
//!   This interaction holds the details of the interaction, including user input and guild details.
//!
//! - `get_contents`
//!   Asynchronously prepares and returns the response content for the stop command execution. The response
//!   is returned as a vector of `EmbedContent` items which can be sent as bot messages. The logic performs
//!   the following steps:
//!
//!   1. **Retrieving Guild ID**:
//!      Extracts the `guild_id` from the command interaction. If no guild ID is available, an error is raised.
//!
//!   2. **Localization Setup**:
//!      Calls `load_localization_stop` to fetch localized strings for constructing the bot's response,
//!      depending on the guild-specific database.
//!
//!   3. **Initializing Lavalink Player**:
//!      It accesses the Lavalink client instance from shared bot data to control the music playback.
//!      - If Lavalink is disabled or uninitialized, it returns an appropriate message.
//!      - If no active audio player (`player`) is found for the guild, it also provides an error response.
//!
//!   4. **Stopping Music Playback**:
//!      Stops the currently playing track (if
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::structure::message::music::stop::load_localization_stop;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing the `StopCommand`, which is used to handle the stop command
/// functionality in a bot using the Serenity library.
///
/// # Fields
///
/// * `ctx` - An instance of `SerenityContext` that contains the context of the bot,
///   including the HTTP client, cache, and framework information.
///
/// * `command_interaction` - An instance of `CommandInteraction` that represents the
///   interaction data for the `stop` command executed by the user.
///
/// # Description
///
/// This struct is typically used to encapsulate the necessary context and data
/// required to process a stop command within the bot's command handling system.
///
/// The `StopCommand` is designed to house all relevant information needed to properly
/// identify the execution context of the command and respond accordingly.
pub struct StopCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for StopCommand {
	/// Retrieves a reference to the Serenity context.
	///
	/// # Returns
	/// A reference to the `SerenityContext` associated with this instance,
	/// allowing access to the context for interacting with the Discord API.
	///
	/// # Examples
	///
	/// ```
	/// let context = instance.get_ctx();
	/// // Use `context` to perform actions such as sending messages or managing events.
	/// ```
	///
	/// This method is particularly useful when you need to access or manipulate
	/// the bot's context outside of predefined event handlers or commands.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` field of the struct.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Now `interaction` holds a reference to the CommandInteraction.
	/// ```
	///
	/// # Note
	/// Ensure that `self.command_interaction` has been properly initialized before calling this method.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes content based on the current command interaction.
	///
	/// This function performs a series of operations, including fetching localized
	/// strings, checking for the presence of a Lavalink client, and interacting
	/// with the audio player's context to stop any currently playing track.
	///
	/// If the bot is not connected to the Lavalink service or no song is currently
	/// playing, appropriate error or informational messages are returned.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` containing the response(s) for the interaction.
	/// - Each `EmbedContent` instance represents a message providing feedback, such
	///   as success, errors, or information.
	/// - `Err(anyhow::Error)` if any error occurs during execution.
	///
	/// # Steps:
	/// 1. Defers the interaction to allow further processing.
	/// 2. Retrieves the guild ID from the command interaction, defaulting to "0" if not present.
	/// 3. Loads localized messages for the guild's language.
	/// 4. Checks if a Lavalink client is available and connected.
	/// 5. Retrieves the player's context for the current guild via Lavalink.
	/// 6. If a track is playing, stops the playback and provides a success message.
	///    Otherwise, provides an appropriate error or informational message.
	///
	/// # Errors:
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
		let stop_localised =
			load_localization_stop(guild_id_str, bot_data.config.db.clone()).await?;

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
				EmbedContent::new(stop_localised.title).description(stop_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

			return Ok(embed_contents);
		};
		let mut embed_content = EmbedContent::new(stop_localised.title);

		let now_playing = player.get_player().await?.track;

		if let Some(np) = now_playing {
			player.stop_now().await?;
			embed_content =
				embed_content.description(stop_localised.success.replace("{0}", &np.info.title));
		} else {
			embed_content = embed_content.description(stop_localised.nothing_to_stop);
		}

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}
