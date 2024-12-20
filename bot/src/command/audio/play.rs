use crate::audio::receiver::{Receiver, TrackEndNotifier, TrackErrorNotifier};
use crate::audio::rusty_ytdl::RustyYoutubeSearch;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::audio::play::load_localization_play_localised;
use anyhow::{anyhow, Result};
use serenity::all::{CommandInteraction, CreateInteractionResponseFollowup};
use serenity::builder::CreateInteractionResponse::Defer;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::prelude::Context as SerenityContext;
use songbird::input::Compose;
use songbird::tracks::Track;
use songbird::{CoreEvent, TrackEvent};
use std::sync::Arc;
use tracing::{error, trace};

pub struct AudioPlayCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AudioPlayCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for AudioPlayCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	let map = get_option_map_string_subcommand(command_interaction);

	let mut url = map
		.get(&String::from("song"))
		.ok_or(anyhow!("No option for song"))?
		.clone();

	let guild_id = command_interaction.guild_id.ok_or(anyhow!("No guild id"))?;

	let cache = ctx.cache.clone();

	let localised =
		load_localization_play_localised(guild_id.to_string(), config.db.clone()).await?;

	let bot_data = ctx.data::<BotData>().clone();

	let manager = bot_data.manager.clone();

	trace!(?manager);

	let builder_message = Defer(CreateInteractionResponseMessage::new());

	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	let bind = manager.get(guild_id);

	trace!(?bind);

	if manager.get(guild_id).is_none() {
		let channel_id;

		{
			let guild = match guild_id.to_guild_cached(&cache) {
				Some(guild) => guild,
				None => {
					error!("Failed to get the guild.");

					return Err(anyhow!("Failed to get the guild"));
				},
			};

			channel_id = guild
				.voice_states
				.get(&command_interaction.user.id)
				.and_then(|voice_state| voice_state.channel_id);
		}

		trace!(?channel_id);

		let connect_to = match channel_id {
			Some(channel) => channel,
			None => return Err(anyhow!("Not connected to a voice channel")),
		};

		let bot_data = ctx.data::<BotData>().clone();

		let manager = bot_data.manager.clone();

		let success = manager.join(guild_id, connect_to).await;

		if let Ok(handler_lock) = success {
			let evt_receiver = Receiver::new();

			let mut handler = handler_lock.lock().await;

			handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::RtpPacket.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::RtcpPacket.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver);
		} else if let Err(joining) = success {
			return Err(anyhow!(format!(
				"Failed to join voice channel: {:#?}",
				joining
			)));
		}
	}

	let bind = manager.get(guild_id);

	trace!(?bind);

	if url.clone().starts_with("http") && url.contains("music.") {
		url = url.replace("music.", "");
	}

	if let Some(handler_mutex) = bind {
		let handler_mutex_clone = handler_mutex.clone();

		let mut handler_lock = handler_mutex_clone.lock().await;

		let do_search = !url.starts_with("http");

		let src = if do_search {
			RustyYoutubeSearch::new_from_search(url.clone())
		} else {
			RustyYoutubeSearch::new_from_url(url.clone())
		};

		let mut src = src?;

		let (_, meta) = futures::join!(
			handler_lock.enqueue(Track::from(src.clone())),
			src.aux_metadata()
		);

		let url = match meta {
			Ok(meta) => {
				let title = meta.title.unwrap_or("song".to_string());

				let thumbnail = meta.thumbnail;

				let duration = meta.duration.unwrap_or_default();

				let mut embed = get_default_embed(None)
					.title(localised.now_playing)
					.description(format!("[{}]({}): {:?}", title, url.clone(), duration));

				if let Some(thumb) = thumbnail {
					embed = embed.thumbnail(thumb);
				}

				let builder = CreateInteractionResponseFollowup::new().embed(embed);

				command_interaction
					.create_followup(&ctx.http, builder)
					.await?;

				meta.source_url.unwrap_or(url)
			},
			Err(_) => url,
		};

		handler_lock.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

		handler_lock.add_global_event(
			TrackEvent::End.into(),
			TrackEndNotifier {
				manager: handler_mutex,
				url,
				guild_id,
			},
		);

		return Ok(());
	}

	let embed = get_default_embed(None).title(localised.error);

	let builder = CreateInteractionResponseFollowup::new().embed(embed);

	command_interaction
		.create_followup(&ctx.http, builder)
		.await?;

	Ok(())
}
