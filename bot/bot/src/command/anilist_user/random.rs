use anyhow::anyhow;
use cynic::{GraphQlResponse, QueryBuilder};
use rand::{rng, RngExt};
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	get_guild_media_scores, get_media, get_registered_anilist_ids,
};
use crate::structure::run::anilist::random::{
	MediaType, RandomPageMedia, RandomPageMediaVariables,
};
use kasuki_macros::slash_command;
use shared::anilist::make_request::make_request_anilist;

#[slash_command(
	name = "random", desc = "Get a random anime or manga.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "type", desc = "Type of the random (anime or manga).", arg_type = String, required = true, autocomplete = false,
		choices = [(name = "anime"), (name = "manga")])],
)]
async fn random_command(self_: RandomCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let random_type = map
		.get(&FixedString::from_str_trunc("type"))
		.ok_or(anyhow!("No type specified"))?;

	let stats = shared::database::random_stats::Entity::find_by_id(1)
		.one(&*cx.db)
		.await?
		.ok_or(anyhow!("No random stats found in database"))?;

	let last_page = if random_type.as_str() == "anime" {
		stats.last_anime_page
	} else if random_type.as_str() == "manga" {
		stats.last_manga_page
	} else {
		0
	};

	trace!(last_page);

	let number = rng().random_range(1..=last_page);

	let mut var = RandomPageMediaVariables {
		media_type: None,
		page: Some(number),
	};

	let shared_media_type = if random_type == "manga" {
		var.media_type = Some(MediaType::Manga);
		Some(shared::anilist::media::MediaType::Manga)
	} else {
		var.media_type = Some(MediaType::Anime);
		Some(shared::anilist::media::MediaType::Anime)
	};

	let operation = RandomPageMedia::build(var);

	let data: GraphQlResponse<RandomPageMedia> =
		make_request_anilist(operation, true, anilist_cache.clone()).await?;

	let data = data.data.ok_or(anyhow!("No data found"))?;

	let id = data
		.page
		.ok_or(anyhow!("No page found"))?
		.media
		.ok_or(anyhow!("No media found"))?
		.get(0)
		.cloned()
		.ok_or(anyhow!("No media found"))?
		.ok_or(anyhow!("No media found"))?
		.id;

	let media_data =
		get_media(&id.to_string(), shared_media_type, None, anilist_cache.clone()).await?;

	let guild_scores = if cx.guild_id != "0" {
		let anilist_ids = get_registered_anilist_ids(&cx.db).await.unwrap_or_default();
		if anilist_ids.is_empty() {
			None
		} else {
			Some(
				get_guild_media_scores(media_data.id, anilist_ids, anilist_cache)
					.await
					.unwrap_or_default(),
			)
		}
	} else {
		None
	};

	let lang_id = cx.lang_id().await;
	let embed_contents =
		media::media_content(media_data, &lang_id, cx.db.clone(), guild_scores).await?;

	Ok(embed_contents)
}
