use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::music::music_context::MusicCommandContext;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "stop", desc = "Stop the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn stop_command(self_: StopCommand) -> Result<EmbedsContents<'_>> {
	let mcx = MusicCommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	)
	.await?;

	let Some(player) = mcx.get_player() else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_stop-title"))
			.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_stop-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};
	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_stop-title"));

	let now_playing = player.get_player().await?.track;

	if let Some(np) = now_playing {
		player.stop_now().await?;
		let args = shared::fluent_args!("var0" => np.info.title.clone());
		embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&mcx.lang_id,
			"music_stop-success",
			&args,
		));
	} else {
		embed_content = embed_content
			.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_stop-nothing_to_stop"));
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
