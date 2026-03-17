use anyhow::anyhow;

use crate::command::anime::random_image::random_image_content;
use crate::command::context::CommandContext;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;

#[slash_command(
	name = "random_himage", desc = "Get a random nsfw anime image.",
	command_type = SubCommand(parent = "random_hanime"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "image_type", desc = "Type of the image you want.", arg_type = String, required = true, autocomplete = false,
		choices = [(name = "waifu"), (name = "neko"), (name = "trap")])],
)]
async fn anime_random_nsfw_image_command(
	self_: AnimeRandomNsfwImageCommand,
) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let image_type = map
		.get("image_type")
		.ok_or_else(|| anyhow!("No image type specified"))?
		.clone();

	let lang_id = cx.lang_id().await;
	let title = USABLE_LOCALES.lookup(&lang_id, "anime_nsfw_random_image_nsfw-title");

	random_image_content(image_type, title, "nsfw").await
}
