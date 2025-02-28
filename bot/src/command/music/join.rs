use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::structure::message::music::join::load_localization_join;
use anyhow::{Result, anyhow};
use lavalink_rs::model::ChannelId;
use serenity::all::{CommandInteraction, Context as SerenityContext, Context};
use serenity::http::Http;
use serenity::prelude::Mentionable;
use std::sync::Arc;

pub struct JoinCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for JoinCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for JoinCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		self.defer().await?;

		let (_, content) = join(ctx, bot_data, command_interaction).await?;

		self.send_embed(content).await
	}
}

pub async fn join<'a>(
	ctx: &'a Context, bot_data: Arc<BotData<'_>>, command_interaction: &'a CommandInteraction,
) -> Result<(bool, EmbedContent<'a, 'a>)> {
	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized strings
	let join_localised = load_localization_join(guild_id_str, bot_data.config.db.clone()).await?;

	let lava_client = bot_data.lavalink.read().await.clone();
	match lava_client {
		Some(_) => {},
		None => {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		},
	}
	let lava_client = lava_client.unwrap();
	let manager = bot_data.manager.clone();

	let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

	// Get the channel information BEFORE creating any futures
	let channel_id = command_interaction.channel_id;
	let author_id = command_interaction.user.id;

	// Extract just the voice channel ID from the guild cache before awaiting anything
	let connect_to = {
		// Extract the voice channel data from the cache
		let guild = guild_id
			.to_guild_cached(&ctx.cache)
			.ok_or(anyhow!("Guild not found"))?;

		let user_channel_id = guild
			.voice_states
			.get(&author_id)
			.and_then(|voice_state| voice_state.channel_id);

		// We only need the channel ID from this scope
		match user_channel_id {
			Some(channel) => channel,
			None => {
				return Ok((
					false,
					EmbedContent {
						title: join_localised.title,
						description: join_localised.error_no_voice,
						thumbnail: None,
						url: None,
						command_type: EmbedType::Followup,
						colour: None,
						fields: vec![],
						images: None,
						action_row: None,
						images_url: None,
					},
				));
			},
		}
	};

	// Create the embed content outside the non-Send guild reference scope
	let mut content = EmbedContent {
		title: join_localised.title,
		description: "".to_string(),
		thumbnail: None,
		url: None,
		command_type: EmbedType::Followup,
		colour: None,
		fields: vec![],
		images: None,
		action_row: None,
		images_url: None,
	};

	if lava_client
		.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		.is_none()
	{
		let handler = manager.join_gateway(guild_id, connect_to).await;

		let (result, return_data) = match handler {
			Ok((connection_info, _)) => {
				lava_client
					.create_player_context_with_data::<(ChannelId, Arc<Http>)>(
						lavalink_rs::model::GuildId::from(guild_id.get()),
						lavalink_rs::model::player::ConnectionInfo {
							endpoint: connection_info.endpoint,
							token: connection_info.token,
							session_id: connection_info.session_id,
						},
						Arc::new((ChannelId(channel_id.get()), ctx.http.clone())),
					)
					.await?;

				content.description = join_localised
					.success
					.replace("{0}", &connect_to.mention().to_string());
				(true, content)
			},
			Err(why) => {
				content.description = join_localised
					.error_joining
					.replace("{0}", &why.to_string());
				(false, content)
			},
		};
		return Ok((result, return_data));
	};
	Ok((false, content))
}
