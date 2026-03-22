use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::music::music_context::MusicCommandContext;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "leave", desc = "Leave the voice channel.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn leave_command(self_: LeaveCommand) -> Result<EmbedsContents<'_>> {
	let mcx = MusicCommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	)
	.await?;

	let manager = mcx.bot_data.manager.clone();

	mcx.lava_client
		.delete_player(lavalink_rs::model::GuildId::from(mcx.guild_id.get()))
		.await?;

	if manager.get(mcx.guild_id).is_some() {
		manager.remove(mcx.guild_id).await?;
	}

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&mcx.lang_id, "music_leave-title"))
		.description(USABLE_LOCALES.lookup(&mcx.lang_id, "music_leave-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
