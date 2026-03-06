use crate::command::server::generate_image_pfp_server::get_content;
use crate::event_handler::BotData;
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
	let ctx = self_.get_ctx().clone();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction().clone();
	let db_connection = bot_data.db_connection.clone();
	let image_store = bot_data.image_store.clone();

	let embed_contents =
		get_content(ctx, command_interaction, "global", db_connection, &image_store).await?;

	Ok(embed_contents)
}
