//! The `SkipCommand` struct represents the functionality of a music skip command in a Discord bot.
//!
//! It implements the `Command` trait to handle retrieving the context and
//! command interaction, deferring responses, and processing the skip logic.
//!
//! The `SkipCommand` retrieves the current playing track, attempts to skip it if possible,
//! and sends appropriate feedback to the user.
//!
//! # Fields
//! - `ctx`: Represents the context in which the command is executed.
//! - `command_interaction`: Represents the interaction associated with this specific command.
//!
//! # Implementations
//! ## `get_ctx`
//! Retrieves the execution context (`SerenityContext`) passed to the command.
//!
//! ## `get_command_interaction`
//! Retrieves the command interaction (`CommandInteraction`) associated with the given command.
//!
//! ## `get_contents`
//! Handles the main logic behind the skip functionality and provides user feedback.
//!
//! This method:
//! 1. Retrieves the guild ID and loads localized strings for the skip command feedback (title, errors, messages).
//! 2. Verifies if the Lavalink client (music client) exists and retrieves the player context for the associated guild.
//! 3. Checks if there is a currently playing track:
//!    - If a track is playing, it skips the track and informs the user of the skipped track's title.
//!    - If no track is playing, it sends feedback indicating there is nothing to skip.
//!
//! ### Errors
//! - Returns an error if:
//!     - Lavalink client is not available or disabled.
//!     - The guild ID cannot be found.
//!     - Playback operations fail or the player context cannot be retrieved.
//!
//! ### Returns
//! - A vector of `EmbedContent` objects containing the message(s) to be sent as bot feedback.
//!
//! # Example Usage
//! When a user sends a `/skip` command:
//! - If a song is currently playing, the bot skips the track and sends a success message.
//! - If no song is playing or the Lavalink client is unavailable, the bot sends an error message.
use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::structure::message::music::skip::load_localization_skip;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// Struct representing the "Skip" command within a Discord bot.
///
/// This command is intended to be used to skip the currently playing track
/// in a music playback session within a Discord voice channel.
///
/// # Fields
///
/// * `ctx`
///   - The context of the Serenity framework, providing access to the Discord client,
///     cache, and HTTP objects.
///   - Used to interact with Discord's API or retrieve information about the bot's state.
///
/// * `command_interaction`
///   - Represents the interaction event related to the "Skip" command.
///   - This contains the details of the command invocation, such as which user
///     issued the command and the specific parameters provided.
///
/// # Example Usage
///
/// ```rust
/// let skip_command = SkipCommand {
///     ctx: serenity_context_instance,
///     command_interaction: command_interaction_instance,
/// };
///
/// // Use skip_command to process the skip action.
/// ```
pub struct SkipCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SkipCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) stored within the instance.
	///
	/// # Example
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use `ctx` as needed
	/// ```
	///
	/// This function is useful when you need to access the shared context
	/// in a Serenity bot to interact with Discord-related operations.
	///
	/// # Note
	/// The lifetime of the returned reference is tied to the lifetime of the instance.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object that is stored within the instance.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use the `interaction` as needed
	/// ```
	///
	/// This method can be used to access the `CommandInteraction` for further processing or inspection.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the contents required for handling a bot command in a guild context.
	///
	/// The function performs the following steps:
	/// - Obtains the contextual data and bot configuration.
	/// - Defers the interaction to allow handling without timeout.
	/// - Gathers guild-specific localized strings for message customization.
	/// - Ensures the LavaLink client is enabled and accessible.
	/// - Checks if a music player is active in the guild and processes the current track.
	///
	/// # Steps
	/// 1. Retrieves the Guild ID from the command interaction. If unavailable, defaults to "0".
	/// 2. Loads the localized strings for the guild to customize user-facing messages.
	/// 3. Confirms the presence of a guild ID in the command interaction. Returns an error if absent.
	/// 4. Ensures that the LavaLink client is active. Returns an error if disabled.
	/// 5. Retrieves the active music player for the given guild. Sends a warning message if no player is found.
	/// 6. Handles the current track:
	///    - If a track is playing, skips it and updates the response with success information.
	///    - If no track is currently playing, informs the user that there's nothing to skip.
	///
	/// # Returns
	/// A `Vec` containing `EmbedContent`:
	/// - On success, a response embed with the result of the skip operation.
	/// - On failure or noteworthy conditions (e.g., no active player or no track to skip), an error or informational embed.
	///
	/// # Errors
	/// Returns an `anyhow::Error` in the following scenarios:
	/// - The guild ID is unavailable in the interaction context.
	/// - The LavaLink client is disabled or inaccessible.
	/// - Issues occur during player retrieval, skipping logic, or localized string loading.
	///
	/// # Dependencies
	/// - Localization provider to load guild-specific messages.
	/// - LavaLink client for interacting with the music player.
	/// - EmbedContent struct to structure the response.
	///
	/// # Example Use
	/// ```ignore
	/// let contents = command_instance.get_contents().await?;
	/// // Process `contents` to construct and send a response to the user.
	/// ```
	///
	/// # Requirements
	/// This function utilizes the following features:
	/// - Async context for LavaLink and interaction deferral handling.
	/// - Localization logic for guild-specific messaging.
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
		let skip_localised =
			load_localization_skip(guild_id_str, bot_data.config.db.clone()).await?;

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
			let embed_content = EmbedContent::new(skip_localised.title)
				.description(skip_localised.error_no_voice)
				.command_type(EmbedType::Followup);
			return Ok(vec![embed_content]);
		};
		let mut embed_content = EmbedContent::new(skip_localised.title).command_type(EmbedType::Followup);

		let now_playing = player.get_player().await?.track;

		if let Some(np) = now_playing {
			player.skip()?;
			embed_content.description = skip_localised.success.replace("{0}", &np.info.title);
		} else {
			embed_content.description = skip_localised.nothing_to_skip;
		}

		Ok(vec![embed_content])
	}
}