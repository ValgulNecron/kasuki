//! This module defines the `RemoveCommand`, a structure and implementation
//! used to handle the "remove" functionality within a bot command interaction.
//! The "remove" command allows users to remove a track from the music queue.
use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};

#[slash_command(
	name = "remove", desc = "Remove a song from the queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [(name = "index", desc = "Index of the song to remove.", arg_type = Integer, required = true, autocomplete = false)],
)]
async fn remove_command(self_: RemoveCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();

	// Retrieve the guild ID from the command interaction
	let guild_id_str = match self_.command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized strings
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

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
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_remove-title"))
				.description(USABLE_LOCALES.lookup(&lang_id, "music_remove-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(command_interaction);

	let index = map.get(&String::from("index")).cloned().unwrap_or_default() as usize;

	player.get_queue().remove(index)?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_remove-title"))
		.description(USABLE_LOCALES.lookup(&lang_id, "music_remove-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
