use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::music::music_context::MusicCommandContext;
use futures::future;
use futures::StreamExt;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "queue", desc = "Show the current queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn queue_command(self_: QueueCommand) -> Result<EmbedsContents<'_>> {
	let mcx = MusicCommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	)
	.await?;

	let Some(player) = mcx.get_player() else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_queue-title"))
			.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_queue-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};
	let queue = player.get_queue();
	let player_data = player.get_player().await?;

	let max = queue.get_count().await?.min(9);

	let requested_by = USABLE_LOCALES.lookup(&mcx.lang_id, "music_queue-requested_by");
	let queue_message = queue
		.enumerate()
		.take_while(|(idx, _)| future::ready(*idx < max))
		.map(|(idx, x)| {
			if let Some(uri) = &x.track.info.uri {
				format!(
					"{} -> [{} - {}](<{}>) | {} <@!{}>",
					idx + 1,
					x.track.info.author,
					x.track.info.title,
					uri,
					requested_by,
					x.track.user_data.unwrap()["requester_id"]
				)
			} else {
				format!(
					"{} -> {} - {} | {} <@!{}",
					idx + 1,
					x.track.info.author,
					x.track.info.title,
					requested_by,
					x.track.user_data.unwrap()["requester_id"]
				)
			}
		})
		.collect::<Vec<_>>()
		.await
		.join("\n");

	let now_playing_message = if let Some(track) = player_data.track {
		let time_s = player_data.state.position / 1000 % 60;
		let time_m = player_data.state.position / 1000 / 60;
		let time = format!("{:02}:{:02}", time_m, time_s);

		let args = shared::fluent_args!(
			"var0" => track.info.author.clone(),
			"var1" => track.info.title.clone(),
			"var2" => track.info.uri.clone().unwrap_or_default(),
			"var3" => time,
			"var4" => format!("<@!{}>", track.user_data.unwrap()["requester_id"]),
		);

		USABLE_LOCALES.lookup_with_args(&mcx.lang_id, "music_queue-now_playing", &args)
	} else {
		USABLE_LOCALES.lookup(&mcx.lang_id, "music_queue-nothing_playing")
	};

	let embed_content = EmbedContent::new(now_playing_message).description(queue_message);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
