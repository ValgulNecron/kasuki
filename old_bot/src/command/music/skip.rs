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
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
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
#[derive(Clone)]
pub struct SkipCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for SkipCommand,
	get_contents = |self_: SkipCommand| async move {
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
		let skip_localised = load_localization_skip(guild_id_str, db_connection).await?;

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
			let embed_content =
				EmbedContent::new(skip_localised.title).description(skip_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

			return Ok(embed_contents);
		};
		let mut embed_content = EmbedContent::new(skip_localised.title);

		let now_playing = player.get_player().await?.track;

		if let Some(np) = now_playing {
			player.skip()?;
			embed_content =
				embed_content.description(skip_localised.success.replace("{0}", &np.info.title));
		} else {
			embed_content = embed_content.description(skip_localised.nothing_to_skip);
		}

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);
