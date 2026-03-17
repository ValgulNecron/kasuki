use crate::command::context::CommandContext;
use crate::command::embed_content::{CommandFiles, EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use image::EncodableLayout;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use uuid::Uuid;

#[slash_command(
	name = "random_image", desc = "Get a random anime image.",
	command_type = SubCommand(parent = "random_anime"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "image_type", desc = "Type of the image you want.", arg_type = String, required = true, autocomplete = false,
		choices = [(name = "waifu"), (name = "neko"), (name = "shinobu"), (name = "megumin"), (name = "bully"), (name = "cuddle"), (name = "cry"), (name = "hug"), (name = "awoo"), (name = "kiss"), (name = "lick"), (name = "pat"), (name = "smug"), (name = "blush"), (name = "smile"), (name = "wave"), (name = "highfive"), (name = "nom"), (name = "bite"), (name = "slap"), (name = "kill"), (name = "kick"), (name = "happy"), (name = "wink"), (name = "dance")])],
)]
async fn anime_random_image_command(self_: AnimeRandomImageCommand) -> Result<EmbedsContents<'_>> {
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
	let title = USABLE_LOCALES.lookup(&lang_id, "anime_random_image-title");

	random_image_content(image_type, title, "sfw").await
}

pub async fn random_image_content<'a>(
	image_type: String, title: String, endpoint: &'a str,
) -> Result<EmbedsContents<'a>> {
	let url = format!("https://api.waifu.pics/{}/{}", endpoint, image_type);

	let resp = reqwest::get(&url).await?;

	let json: serde_json::Value = resp.json().await?;

	let image_url = json["url"]
		.as_str()
		.ok_or(anyhow!("No image found"))?
		.to_string();

	let response = reqwest::get(image_url).await?;

	let bytes = response.bytes().await?;

	let uuid_name = Uuid::new_v4();

	let filename = format!("{}.gif", uuid_name);

	let bytes = bytes.as_bytes().to_vec();
	let file = CommandFiles::new(filename.clone(), bytes);

	let embed_content =
		EmbedContent::new(title).images_url(format!("attachment://{}", filename.clone()));

	let embed_contents = EmbedsContents::new(vec![embed_content]).add_files(vec![file]);

	Ok(embed_contents)
}
