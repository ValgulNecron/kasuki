use crate::command::context::CommandContext;
use anyhow::anyhow;
use lavalink_rs::client::LavalinkClient;
use lavalink_rs::player_context::PlayerContext;
use serenity::all::{CommandInteraction, Context as SerenityContext, GuildId};
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

/// Common setup for music commands: extracts guild_id, validates lava_client,
/// and provides player access.
pub struct MusicCommandContext {
	pub cx: CommandContext,
	pub lang_id: LanguageIdentifier,
	pub guild_id: GuildId,
	pub lava_client: Arc<LavalinkClient>,
}

impl MusicCommandContext {
	pub async fn new(
		ctx: SerenityContext, command_interaction: CommandInteraction,
	) -> anyhow::Result<Self> {
		let cx = CommandContext::new(ctx, command_interaction);
		let lang_id = cx.lang_id().await;
		let guild_id = cx
			.command_interaction
			.guild_id
			.ok_or(anyhow!("no guild id"))?;
		let lava_client = cx
			.bot_data
			.lavalink
			.read()
			.await
			.clone()
			.ok_or(anyhow!("Lavalink is disabled"))?;
		Ok(Self {
			cx,
			lang_id,
			guild_id,
			lava_client,
		})
	}

	pub fn get_player(&self) -> Option<PlayerContext> {
		self.lava_client
			.get_player_context(lavalink_rs::model::GuildId::from(self.guild_id.get()))
	}
}

impl std::ops::Deref for MusicCommandContext {
	type Target = CommandContext;

	fn deref(&self) -> &CommandContext {
		&self.cx
	}
}
