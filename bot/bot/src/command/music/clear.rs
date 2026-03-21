use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "clear", desc = "Clear the current queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn clear_command(self_: ClearCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("no guild id"))?;

	let lang_id = cx.lang_id().await;

	let lava_client = cx.bot_data.lavalink.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();

	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_clear-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_clear-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	player.get_queue().clear()?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_clear-title"))
		.description(USABLE_LOCALES.lookup(&lang_id, "music_clear-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);
	Ok(embed_contents)
}
