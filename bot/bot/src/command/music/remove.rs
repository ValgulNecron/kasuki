//! This module defines the `RemoveCommand`, a structure and implementation
//! used to handle the "remove" functionality within a bot command interaction.
//! The "remove" command allows users to remove a track from the music queue.
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "remove", desc = "Remove a song from the queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [(name = "index", desc = "Index of the song to remove.", arg_type = Integer, required = true, autocomplete = false)],
)]
async fn remove_command(self_: RemoveCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	// Load the localized strings
	let lang_id = cx.lang_id().await;

	let guild_id = cx.command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
	let lava_client = cx.bot_data.lavalink.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();
	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_remove-title"))
				.description(USABLE_LOCALES.lookup(&lang_id, "music_remove-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(&cx.command_interaction);

	let index = map.get("index").cloned().unwrap_or_default() as usize;

	player.get_queue().remove(index)?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_remove-title"))
		.description(USABLE_LOCALES.lookup(&lang_id, "music_remove-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
