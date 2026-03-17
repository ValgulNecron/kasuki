use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[slash_command(
	name = "user", desc = "Get info of a VN user.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username of the VN user.", arg_type = String, required = true, autocomplete = false)],
)]
async fn vn_user_command(self_: VnUserCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let username = map
		.get("username")
		.ok_or(anyhow!("No username provided"))?;

	let lang_id = cx.lang_id().await;

	let result = shared::service::vndb::lookup_user(
		username,
		&lang_id,
		cx.vndb_cache.clone(),
		&cx.bot_data.http_client,
	)
	.await?;

	let embed_content = EmbedContent::new(result.title).fields(result.fields);

	Ok(EmbedsContents::new(vec![embed_content]))
}
