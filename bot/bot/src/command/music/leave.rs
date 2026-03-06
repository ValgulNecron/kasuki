//! This module defines and implements the `LeaveCommand` structure, which is a concrete
//! implementation of the `Command` trait. The purpose of the `LeaveCommand` is to handle
//! the "leave" command functionality, primarily for managing the bot's disconnection
//! from voice channels in a guild context.
use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};

#[slash_command(
	name = "leave", desc = "Leave the voice channel.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn leave_command(self_: LeaveCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();

	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized strings
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

	let manager = bot_data.manager.clone();
	let lava_client = bot_data.lavalink.clone();
	let lava_client = lava_client.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

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
