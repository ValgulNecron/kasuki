use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::music::music_context::MusicCommandContext;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use std::time::Duration;

#[slash_command(
	name = "seek", desc = "Seek to a position in the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [(name = "time", desc = "Time to seek to in seconds.", arg_type = Integer, required = true, autocomplete = false)],
)]
async fn seek_command(self_: SeekCommand) -> Result<EmbedsContents<'_>> {
	let mcx = MusicCommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	)
	.await?;

	let Some(player) = mcx.get_player() else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_seek-title"))
			.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_seek-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(&mcx.command_interaction);

	let time = map.get("time").cloned().unwrap_or_default() as u64;

	let now_playing = player.get_player().await?.track;

	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_seek-title"));

	if let Some(_) = now_playing {
		player.set_position(Duration::from_secs(time)).await?;
		let args = shared::fluent_args!("var0" => time.to_string());
		embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&mcx.lang_id,
			"music_seek-success",
			&args,
		));
	} else {
		embed_content = embed_content
			.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_seek-nothing_playing"));
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
