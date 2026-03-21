use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	get_guild_media_scores, get_media, get_registered_anilist_ids, MediaFormat, MediaType,
};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

#[slash_command(
	name = "ln", desc = "Info of a light novel.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "ln_name", desc = "Name of the light novel you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn ln_command(self_: LnCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();
	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("ln_name"))
		.cloned()
		.unwrap_or(String::new());

	let format_in = Some(vec![Some(MediaFormat::Novel)]);
	let data = get_media(
		&value,
		Some(MediaType::Manga),
		format_in,
		anilist_cache.clone(),
	)
	.await?;

	let guild_scores = if cx.guild_id != "0" {
		let anilist_ids = get_registered_anilist_ids(&cx.db).await.unwrap_or_default();
		if anilist_ids.is_empty() {
			None
		} else {
			Some(
				get_guild_media_scores(data.id, anilist_ids, anilist_cache)
					.await
					.unwrap_or_default(),
			)
		}
	} else {
		None
	};

	let lang_id = cx.lang_id().await;
	let embed_content = media::media_content(data, &lang_id, cx.db.clone(), guild_scores).await?;

	Ok(embed_content)
}
