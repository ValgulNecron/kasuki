use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[slash_command(
	name = "producer", desc = "Get info of a VN producer.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the producer.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_producer_command(self_: VnProducerCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let name = map.get("name").cloned().unwrap_or_default();

	let lang_id = cx.lang_id().await;

	let result = shared::service::vndb::lookup_producer(
		name,
		&lang_id,
		cx.vndb_cache.clone(),
		&cx.bot_data.http_client,
	)
	.await?;

	let embed_content = EmbedContent::new(result.name)
		.description(result.description.unwrap_or_default())
		.fields(result.fields)
		.url(result.url);

	Ok(EmbedsContents::new(vec![embed_content]))
}
