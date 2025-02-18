use crate::audio::receiver::Receiver;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::audio::join::load_localization_join_localised;
use anyhow::{anyhow, Result};
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateEmbed};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use songbird::CoreEvent;
use std::sync::Arc;
use tracing::{error, trace};

pub struct AudioJoinCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AudioJoinCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for AudioJoinCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.command_interaction.clone();
		let guild_id = command_interaction.guild_id.ok_or(anyhow!("No guild id"))?;

		let manager = ctx.data::<BotData>().manager.clone();

		let bind = manager.get(guild_id);
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


	trace!(?bind);

	let cache = ctx.cache.clone();

	let localised =
		load_localization_join_localised(guild_id.to_string(), config.db.clone()).await?;

	if manager.get(guild_id).is_none() {
		let channel_id;

		{
			let guild = match guild_id.to_guild_cached(&cache) {
				Some(guild) => guild,
				None => {
					error!("Failed to get the guild.");

					return Err(anyhow!("Failed to get the guild."));
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

		let manager = ctx.data::<BotData>().manager.clone();

		let success = manager.join(guild_id, connect_to).await;

		if let Ok(handler_lock) = success {
			let evt_receiver = Receiver::new();

			let mut handler = handler_lock.lock().await;

			handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::RtpPacket.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::RtcpPacket.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver.clone());

			handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver);

			let embed = CreateEmbed::new().title(localised.title);

			let builder_embed = CreateInteractionResponseMessage::new().embed(embed);

			let builder = CreateInteractionResponse::Message(builder_embed);

			command_interaction
				.create_response(&ctx.http, builder)
				.await?;

			return Ok(());
		} else if let Err(joining) = success {
			return Err(anyhow!(format!(
				"Failed to join voice channel: {:#?}",
				joining
			)));
		}

		Ok(())
	} else {
		let embed = get_default_embed(None).title(localised.already_in);

		let builder_embed = CreateInteractionResponseMessage::new().embed(embed);

		let builder = CreateInteractionResponse::Message(builder_embed);

		command_interaction
			.create_response(&ctx.http, builder)
			.await?;

		Ok(())
	}
}
