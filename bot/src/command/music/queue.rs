//! Documentation for QueueCommand and associated functionality
use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::structure::message::music::queue::load_localization_queue;
use anyhow::anyhow;
use futures::StreamExt;
use futures::future;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// Represents a command handler for managing queue-related interactions in a bot.
///
/// This struct is used to handle user commands in the context of a music or task queue system,
/// with an instance of `SerenityContext` and `CommandInteraction` provided for processing user interactions.
///
/// # Fields
/// - `ctx: SerenityContext`
///   The context provided by the Serenity framework, used for interacting with the bot's state,
///   such as sending messages, accessing guild data, and interacting with other Discord API features.
///
/// - `command_interaction: CommandInteraction`
///   Represents the slash command interaction received from a user, including the command name,
///   parameters, the originating user, and the channel or guild in which the command was invoked.
///
/// # Example Usage
/// ```rust
/// // Example initialization of QueueCommand
/// use serenity::prelude::*;
/// use serenity::model::application::interaction::application_command::CommandInteraction;
///
/// let ctx = SerenityContext::default();
/// let interaction = CommandInteraction {
///     // populate with necessary data
/// };
///
/// let queue_command = QueueCommand { ctx, command_interaction: interaction };
/// // Handle the command here
/// ```
///
/// # Notes
/// - This struct assumes you're using the `serenity` crate and handling slash commands.
/// - The `ctx` and `command_interaction` fields are typically passed to functions that execute the logic for specific queue commands.
///
pub struct QueueCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for QueueCommand {
	/// Retrieves a reference to the `SerenityContext`.
	///
	/// # Returns
	/// A reference to the `SerenityContext` stored in the current instance.
	///
	/// # Examples
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` for further processing
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `command_interaction` associated with this instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object stored in this instance.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// ```
	///
	/// # Note
	/// This function provides read-only access to the `command_interaction`
	/// and does not allow for modifications.
	///
	/// # See Also
	/// - `CommandInteraction` for more details on its structure and usage.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronous function to retrieve and prepare embed content containing the current state
	/// of the music player queue for a Discord bot using Lavalink.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` on success: A vector of embed content with details of the
	///   music queue, including now-playing information and queued tracks.
	/// - `Err(anyhow::Error)` on failure: If there are any errors in retrieving data,
	///   localization, or interactions with Lavalink.
	///
	/// # Errors
	/// This function may return an error in the following cases:
	/// - If localization data cannot be loaded (e.g., database errors).
	/// - If the bot fails to retrieve the guild ID or Lavalink is disabled.
	/// - If no player or player context is available for the associated guild.
	///
	/// # Details
	/// 1. The function first retrieves the application context and bot data.
	/// 2. Loads localized strings for queue-related messages based on the guild ID.
	/// 3. Confirms Lavalink is enabled and retrieves the player context for the guild.
	/// 4. Constructs an embed message with:
	///    a. A "now playing" message detailing the current track (if any).
	///    b. A formatted list of tracks currently in the queue.
	/// 5. If no player is present, or the bot is not in a voice channel, an appropriate error
	///    message is returned.
	///
	/// # Localization
	/// The function uses localized string templates for all user-facing messages to adapt content
	/// based on the guild's configuration or language.
	///
	/// # Example Usage
	/// This function is typically used in the command-handling flow of the bot to return music
	/// queue details to the user.
	///
	/// # Dependencies
	/// - `lavalink_rs`: For managing the music player and queue.
	/// - `anyhow`: For error handling.
	/// - `EmbedContent`: A custom structure for formatting and returning embed content.
	/// - `load_localization_queue`: A function for loading localized strings.
	///
	/// # Example
	/// ```rust
	/// let embed_contents = my_handler.get_contents().await?;
	/// for embed_content in embed_contents {
	///     println!("Title: {}", embed_content.title);
	///     println!("Description: {}", embed_content.description);
	/// }
	/// ```
	///
	/// # Notes
	/// - The function ensures the bot's state and player's context are properly validated before
	///   attempting to retrieve or format the music queue.
	/// - A maximum of 9 tracks are included in the queue message to limit excessive output.
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
		let queue_localised =
			load_localization_queue(guild_id_str, bot_data.config.db.clone()).await?;

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
			let embed_content = EmbedContent::new(queue_localised.title)
				.description(queue_localised.error_no_voice)
				.command_type(EmbedType::Followup);
			return Ok(vec![embed_content]);
		};
		let queue = player.get_queue();
		let player_data = player.get_player().await?;

		let max = queue.get_count().await?.min(9);

		let queue_message = queue
			.enumerate()
			.take_while(|(idx, _)| future::ready(*idx < max))
			.map(|(idx, x)| {
				if let Some(uri) = &x.track.info.uri {
					format!(
						"{} -> [{} - {}](<{}>) | {} <@!{}>",
						idx + 1,
						x.track.info.author,
						x.track.info.title,
						uri,
						queue_localised.requested_by,
						x.track.user_data.unwrap()["requester_id"]
					)
				} else {
					format!(
						"{} -> {} - {} | {} <@!{}",
						idx + 1,
						x.track.info.author,
						x.track.info.title,
						queue_localised.requested_by,
						x.track.user_data.unwrap()["requester_id"]
					)
				}
			})
			.collect::<Vec<_>>()
			.await
			.join("\n");

		let now_playing_message = if let Some(track) = player_data.track {
			let time_s = player_data.state.position / 1000 % 60;
			let time_m = player_data.state.position / 1000 / 60;
			let time = format!("{:02}:{:02}", time_m, time_s);

			if let Some(uri) = &track.info.uri {
				queue_localised
					.now_playing
					.replace("{0}", &track.info.author)
					.replace("{1}", &track.info.title)
					.replace("{2}", uri)
					.replace("{3}", &time)
					.replace(
						"{4}",
						&format!("<@!{}>", track.user_data.unwrap()["requester_id"]),
					)
					.to_string()
			} else {
				queue_localised
					.now_playing
					.replace("{0}", &track.info.author)
					.replace("{1}", &track.info.title)
					.replace("{2}", "")
					.replace("{3}", &time)
					.replace(
						"{4}",
						&format!("<@!{}>", track.user_data.unwrap()["requester_id"]),
					)
					.to_string()
			}
		} else {
			queue_localised.nothing_playing.clone()
		};

		let embed_content = EmbedContent::new(now_playing_message)
			.description(queue_message)
			.command_type(EmbedType::Followup);

		Ok(vec![embed_content])
	}
}
