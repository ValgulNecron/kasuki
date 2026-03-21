use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

#[slash_command(
	name = "seek", desc = "Seek to a position in the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [(name = "time", desc = "Time to seek to in seconds.", arg_type = Integer, required = true, autocomplete = false)],
)]
async fn seek_command(self_: SeekCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let lang_id = cx.lang_id().await;

	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("no guild id"))?;

	let lava_client = cx.bot_data.lavalink.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();
	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_seek-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_seek-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(&cx.command_interaction);

	let time = map.get("time").cloned().unwrap_or_default() as u64;

	let now_playing = player.get_player().await?.track;

	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_seek-title"));

	if let Some(_) = now_playing {
		player.set_position(Duration::from_secs(time)).await?;
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("var0"), FluentValue::from(time.to_string()));
		embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"music_seek-success",
			&args,
		));
	} else {
		embed_content = embed_content
			.description(USABLE_LOCALES.lookup(&lang_id, "music_seek-nothing_playing"));
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
