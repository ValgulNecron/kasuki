use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[slash_command(
	name = "stats", desc = "Get VN statistics.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn vn_stats_command(self_: VnStatsCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let lang_id = cx.lang_id().await;

	let result = shared::service::vndb::lookup_stats(
		&lang_id,
		cx.vndb_cache.clone(),
		&cx.bot_data.http_client,
	)
	.await?;

	let embed_content = EmbedContent::new(result.title).fields(result.fields);

	Ok(EmbedsContents::new(vec![embed_content]))
}
