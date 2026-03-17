use crate::command::context::CommandContext;
use crate::command::server::generate_image_pfp_server::get_content;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[slash_command(
	name = "guild_image_g", desc = "Generate global profile picture for the guild.",
	command_type = SubCommand(parent = "server"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn generate_global_image_pfp_command(
	self_: GenerateGlobalImagePfPCommand,
) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let embed_contents =
		get_content(cx.ctx.clone(), cx.command_interaction.clone(), "global", cx.db.clone(), &cx.image_store).await?;

	Ok(embed_contents)
}
