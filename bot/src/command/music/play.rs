//! The `PlayCommand` struct encapsulates functionality needed to execute a
//! "play" command in a Discord bot application, leveraging the Lavalink library for music playback.
//!
//! This struct is responsible for handling user interactions, parsing the provided inputs,
//! connecting a bot instance to a voice channel, and enqueuing audio tracks for playback.
//!
//! # Fields
//! - `ctx`:
//!   The context of the current command, which provides access to bot state and libraries
//!   for interacting with Discord.
//! - `command_interaction`:
//!   Represents the specific command interaction invoked by a user, containing input data, options,
//!   and the guild-related state.
//!
//! This struct implements the `Command
use crate::command::command::{Command, CommandRun, EmbedContent};
use crate::command::music::join::join;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::music::play::load_localization_play;
use anyhow::anyhow;
use lavalink_rs::player_context::TrackInQueue;
use lavalink_rs::prelude::{SearchEngines, TrackLoadData};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::trace;

/// Represents a command to play music or audio within a Discord bot using the `serenity` framework.
///
/// The `PlayCommand` struct encapsulates the necessary context and command interaction information
/// required to execute a "play" command for the bot.
///
/// # Fields
///
/// * `ctx` - The [`SerenityContext`](https://docs.rs/serenity/latest/serenity/prelude/struct.Context.html)
///   provided by the `serenity` framework, which contains information about the bot's state and functionality.
///   This is required to interact with Discord services and manage the
pub struct PlayCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for PlayCommand {
	/// Returns a reference to the `SerenityContext` associated with the current instance.
	///
	/// The `SerenityContext` provides access to various features and utilities of the Serenity library,
	/// including interacting with Discord's API, retrieving data about the environment, or managing
	/// the bot's state.
	///
	/// # Returns
	/// - A reference to the `SerenityContext` used by this instance.
	///
	/// # Example
	/// ```rust
	/// let ctx = instance.get
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` stored within this object.
	///
	/// # Example
	/// ```rust
	/// let interaction = object.get_command_interaction();
	/// // Now you can use the `interaction` reference.
	/// ```
	///
	/// # Notes
	/// - The returned reference has the same lifetime as the object on which the method is called.
	///
	/// # Safety
	/// This method assumes that `self.command_interaction` has been
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves contents based on user interaction in a bot command.
	///
	/// This asynchronous function handles processing for a command by deferring its execution,
	/// retrieving necessary context, and interacting with external systems such as a Lavalink music
	/// player client. It performs the following steps:
	///
	/// 1. Retrieve the command's context and bot-specific data.
	/// 2. Defer the command execution to acknowledge interaction timely.
	/// 3. Extract the guild ID from the command interaction and fetch localization settings.
	/// 4. Validate and utilize the Lavalink client for music-related functionalities.
	/// 5. Retrieve the search query input
	async fn get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let play_localised =
			load_localization_play(guild_id_str, bot_data.config.db.clone()).await?;

		let lava_client = bot_data.lavalink.read().await.clone();
		let (_, mut embed_content) = join(ctx, bot_data, command_interaction).await?;

		match lava_client {
			Some(_) => {},
			None => {
				return Err(anyhow::anyhow!("Lavalink is disabled"));
			},
		}
		let lava_client = lava_client.unwrap();
		let command_interaction = self.get_command_interaction();
		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

		let Some(player) =
			lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		else {
			embed_content[0] = embed_content[0]
				.clone()
				.description(play_localised.error_no_voice);
			return Ok(embed_content);
		};

		let map = get_option_map_string_subcommand(command_interaction);
		trace!("{:#?}", map);
		let term = map
			.get(&String::from("search"))
			.cloned()
			.unwrap_or_default();
		trace!(term);

		let query = if term.starts_with("http") {
			term
		} else {
			SearchEngines::YouTube.to_query(&term)?
		};

		let loaded_tracks = lava_client
			.load_tracks(lavalink_rs::model::GuildId::from(guild_id.get()), &query)
			.await?;

		let mut playlist_info = None;

		let mut tracks: Vec<TrackInQueue> = match loaded_tracks.data {
			Some(TrackLoadData::Track(x)) => vec![x.into()],
			Some(TrackLoadData::Search(x)) => vec![x[0].clone().into()],
			Some(TrackLoadData::Playlist(x)) => {
				playlist_info = Some(x.info);
				x.tracks.iter().map(|x| x.clone().into()).collect()
			},

			_ => {
				embed_content[0] = embed_content[0]
					.clone()
					.description(format!("{:?}", loaded_tracks));
				return Ok(embed_content);
			},
		};

		if let Some(info) = playlist_info {
			embed_content[0] = embed_content[0]
				.clone()
				.description(play_localised.added_playlist.replace("{0}", &info.name));
		} else {
			let track = &tracks[0].track;

			if let Some(uri) = &track.info.uri {
				embed_content[0] = embed_content[0].clone().description(
					play_localised
						.added_to_queue
						.replace("{0}", &track.info.author)
						.replace("{1}", &track.info.title)
						.replace("{2}", uri),
				);
			} else {
				embed_content[0] = embed_content[0].clone().description(
					play_localised
						.added_to_queue
						.replace("{0}", &track.info.author)
						.replace("{1}", &track.info.title)
						.replace("{2}", ""),
				);
			}

			return Ok(embed_content);
		}

		let author_id = command_interaction.user.id;
		for i in &mut tracks {
			i.track.user_data = Some(serde_json::json!({"requester_id": author_id}));
		}

		let queue = player.get_queue();
		queue.append(tracks.into())?;

		if let Ok(player_data) = player.get_player().await {
			if player_data.track.is_none() && queue.get_track(0).await.is_ok_and(|x| x.is_some()) {
				player.skip()?;
			}
		}

		Ok(embed_content)
	}
}
