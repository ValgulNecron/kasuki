use crate::command::context::CommandContext;
use crate::command::embed_content::EmbedsContents;
use crate::command::vn::build_vn_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[slash_command(
	name = "character", desc = "Get info of a VN character.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the character.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_character_command(self_: VnCharacterCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let name = map.get("name").cloned().unwrap_or_default();

	let lang_id = cx.lang_id().await;

	let result = shared::service::vndb::lookup_character(
		name,
		&lang_id,
		cx.vndb_cache.clone(),
		&cx.bot_data.http_client,
	)
	.await?;

	Ok(build_vn_embed(
		result.name,
		result.description,
		result.fields,
		result.url,
		result.image_url,
	))
}
