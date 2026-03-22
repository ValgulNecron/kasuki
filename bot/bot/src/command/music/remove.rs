use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::music::music_context::MusicCommandContext;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
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
	let mcx = MusicCommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	)
	.await?;

	let Some(player) = mcx.get_player() else {
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_remove-title"))
				.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_remove-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(&mcx.command_interaction);

	let index = map.get("index").cloned().unwrap_or_default() as usize;

	player.get_queue().remove(index)?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_remove-title"))
		.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_remove-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
