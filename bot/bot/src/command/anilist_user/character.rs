use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::character;
use crate::structure::run::anilist::character::get_character;
use anyhow::{Context, Result};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

#[slash_command(
	name = "character", desc = "Info of a character.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the character you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn character_command(self_: CharacterCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);
	let value = map
		.get(&FixedString::from_str_trunc("name"))
		.cloned()
		.unwrap_or(String::new());

	let data = get_character(&value, anilist_cache).await?;
	let lang_id = cx.lang_id().await;
	let embed_contents = character::character_content(data, &lang_id)
		.await
		.context("Failed to generate character content for embed")?;

	Ok(embed_contents)
}
