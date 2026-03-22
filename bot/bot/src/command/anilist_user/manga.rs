use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{fetch_guild_scores, get_media, MediaFormat, MediaType};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

#[slash_command(
	name = "manga", desc = "Info of a manga.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "manga_name", desc = "Name of the manga you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn manga_command(self_: MangaCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();
	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("manga_name"))
		.cloned()
		.unwrap_or(String::new());

	let format_in = Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]);
	let data = get_media(
		&value,
		Some(MediaType::Manga),
		format_in,
		anilist_cache.clone(),
	)
	.await?;

	let member_ids = cx.guild_member_ids();
	let guild_scores =
		fetch_guild_scores(&member_ids, data.id, &cx.db, anilist_cache).await;

	let lang_id = cx.lang_id().await;
	let embed_content = media::media_content(data, &lang_id, cx.db.clone(), guild_scores).await?;

	Ok(embed_content)
}
