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
use crate::command::command::CommandRun;
use crate::command::music::join::join;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use anyhow::{anyhow, Context};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use lavalink_rs::player_context::TrackInQueue;
use lavalink_rs::prelude::{SearchEngines, TrackLoadData};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::trace;

#[slash_command(
	name = "play", desc = "Play a song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [(name = "search", desc = "Search for a song.", arg_type = String, required = true, autocomplete = false)],
)]
async fn play_command(self_: PlayCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx().clone();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction().clone();

	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized strings
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

	let lava_client = bot_data.lavalink.read().await.clone();
	let (_, mut embed_content) = join(ctx, bot_data, command_interaction).await?;

	match lava_client {
		Some(_) => {},
		None => {
			return Err(anyhow::anyhow!("Lavalink is disabled")).with_context(|| {
				"Cannot play music because Lavalink service is not configured or unavailable"
			});
		},
	}
	let lava_client = lava_client.unwrap();
	let command_interaction = self_.get_command_interaction();
	let guild_id = command_interaction
		.guild_id
		.ok_or(anyhow!("no guild id"))
		.with_context(|| {
			"Command must be used in a server, not in DMs or other non-guild contexts"
		})?;

	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		embed_content.embed_contents[0] = embed_content.embed_contents[0]
			.clone()
			.description(USABLE_LOCALES.lookup(&lang_id, "music_play-error_no_voice"));
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
			embed_content.embed_contents[0] = embed_content.embed_contents[0]
				.clone()
				.description(format!("{:?}", loaded_tracks));
			return Ok(embed_content);
		},
	};

	if let Some(info) = playlist_info {
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("var0"), FluentValue::from(info.name.clone()));

		embed_content.embed_contents[0] =
			embed_content.embed_contents[0]
				.clone()
				.description(USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"music_play-added_playlist",
					&args,
				));
	} else {
		let track = &tracks[0].track;

		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(
			Cow::Borrowed("var0"),
			FluentValue::from(track.info.author.clone()),
		);
		args.insert(
			Cow::Borrowed("var1"),
			FluentValue::from(track.info.title.clone()),
		);
		args.insert(
			Cow::Borrowed("var2"),
			FluentValue::from(track.info.uri.clone().unwrap_or_default()),
		);

		embed_content.embed_contents[0] =
			embed_content.embed_contents[0]
				.clone()
				.description(USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"music_play-added_to_queue",
					&args,
				));

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
