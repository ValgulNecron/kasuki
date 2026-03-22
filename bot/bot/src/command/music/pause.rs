use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::music::music_context::MusicCommandContext;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "pause", desc = "Pause the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn pause_command(self_: PauseCommand) -> Result<EmbedsContents<'_>> {
	let mcx = MusicCommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	)
	.await?;

	let Some(player) = mcx.get_player() else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_pause-title"))
			.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_pause-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);
		return Ok(embed_contents);
	};
	player.set_pause(true).await?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_pause-title"))
		.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_pause-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);
	Ok(embed_contents)
}
