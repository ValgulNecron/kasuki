//! This module defines and implements the `LeaveCommand` structure, which is a concrete
//! implementation of the `Command` trait. The purpose of the `LeaveCommand` is to handle
//! the "leave" command functionality, primarily for managing the bot's disconnection
//! from voice channels in a guild context.
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "leave", desc = "Leave the voice channel.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn leave_command(self_: LeaveCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	// Load the localized strings
	let lang_id = cx.lang_id().await;

	let manager = cx.bot_data.manager.clone();
	let lava_client = cx.bot_data.lavalink.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("no guild id"))?;

	let lava_client = lava_client.unwrap();

	lava_client
		.delete_player(lavalink_rs::model::GuildId::from(guild_id.get()))
		.await?;

	if manager.get(guild_id).is_some() {
		manager.remove(guild_id).await?;
	}

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_leave-title"))
		.description(USABLE_LOCALES.lookup(&lang_id, "music_leave-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
