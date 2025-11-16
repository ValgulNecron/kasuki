//! Documentation for QueueCommand and associated functionality
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::music::queue::load_localization_queue;
use anyhow::anyhow;
use futures::future;
use futures::StreamExt;
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
#[derive(Clone)]
pub struct QueueCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for QueueCommand,
	get_contents = |self_: QueueCommand| async move {
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
		let queue_localised = load_localization_queue(guild_id_str, db_connection).await?;

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
			let embed_content = EmbedContent::new(queue_localised.title)
				.description(queue_localised.error_no_voice);

			let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

			return Ok(embed_contents);
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

		let embed_content = EmbedContent::new(now_playing_message).description(queue_message);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);
