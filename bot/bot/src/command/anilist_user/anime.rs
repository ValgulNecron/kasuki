use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{get_media, MediaFormat, MediaType};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

#[slash_command(
	name = "anime", desc = "Info of an anime.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "anime_name", desc = "Name of the anime you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn anime_command(self_: AnimeCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string(&cx.command_interaction);
	let value = map
		.get(&FixedString::from_str_trunc("anime_name"))
		.cloned()
		.unwrap_or(String::new());

	let format_in = Some(vec![
		Some(MediaFormat::Tv),
		Some(MediaFormat::TvShort),
		Some(MediaFormat::Movie),
		Some(MediaFormat::Special),
		Some(MediaFormat::Ova),
		Some(MediaFormat::Ona),
		Some(MediaFormat::Music),
	]);

	let anilist_cache = cx.anilist_cache.clone();

	let data = get_media(&value, Some(MediaType::Anime), format_in, anilist_cache).await?;
	let lang_id = cx.lang_id().await;
	let embed_contents = media::media_content(data, &lang_id, cx.db.clone()).await?;

	Ok(embed_contents)
}
