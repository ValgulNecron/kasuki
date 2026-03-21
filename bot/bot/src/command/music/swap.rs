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

#[slash_command(
	name = "swap", desc = "Swap two songs in the queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [
		(name = "index1", desc = "Index of the first song.", arg_type = Integer, required = true, autocomplete = false),
		(name = "index2", desc = "Index of the second song.", arg_type = Integer, required = true, autocomplete = false)
	],
)]
async fn swap_command(self_: SwapCommand) -> Result<EmbedsContents<'_>> {
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
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_swap-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_swap-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(&cx.command_interaction);

	let index1 = map.get("index1").cloned().unwrap_or_default() as usize;

	let index2 = map.get("index2").cloned().unwrap_or_default() as usize;

	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_swap-title"));

	let queue = player.get_queue();
	let queue_len = queue.get_count().await?;

	if index1 > queue_len || index2 > queue_len {
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(
			Cow::Borrowed("var0"),
			FluentValue::from(queue_len.to_string()),
		);
		embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"music_swap-error_max_index",
			&args,
		));
	} else if index1 == index2 {
		embed_content = embed_content
			.description(USABLE_LOCALES.lookup(&lang_id, "music_swap-error_same_index"));
	} else {
		let track1 = queue.get_track(index1 - 1).await?.unwrap();
		let track2 = queue.get_track(index1 - 2).await?.unwrap();

		queue.swap(index1 - 1, track2)?;
		queue.swap(index2 - 1, track1)?;

		embed_content =
			embed_content.description(USABLE_LOCALES.lookup(&lang_id, "music_swap-success"));
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
