//! This module defines the `MangaCommand` structure and its implementation for fetching manga
//! information using the AniList GraphQL API. It forms part of a bot system based on the Serenity library
//! for managing commands and interactions.
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
	MediaQuerrySearchVariables, MediaType,
};
use cynic::{GraphQlResponse, QueryBuilder};
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
	let ctx = self_.get_ctx().clone();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction().clone();

	let anilist_cache = bot_data.anilist_cache.clone();
	let _config = bot_data.config.clone();
	let map = get_option_map_string(&command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("manga_name"))
		.cloned()
		.unwrap_or(String::new());

	// Fetch the manga data by ID if the value can be parsed as an `i32`, or by search otherwise
	let data: Media = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>()?;

		let var = MediaQuerryIdVariables {
			format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
			id: Some(id),
			media_type: Some(MediaType::Manga),
		};

		let operation = MediaQuerryId::build(var);

		let data: GraphQlResponse<MediaQuerryId> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().media.unwrap()
	} else {
		let var = MediaQuerrySearchVariables {
			format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
			search: Some(&*value),
			media_type: Some(MediaType::Manga),
		};

		let operation = MediaQuerrySearch::build(var);

		let data: GraphQlResponse<MediaQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().media.unwrap()
	};
	let db_connection = bot_data.db_connection.clone();

	let embed_content =
		media::media_content(ctx, command_interaction, data, db_connection, bot_data).await?;

	Ok(embed_content)
}
