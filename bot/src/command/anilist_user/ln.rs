//! Represents the `LnCommand` structure, which implements the `Command` trait
//! and handles interactions for fetching Light Novel (LN) data from AniList.
//!
//! # Fields
//! * `ctx` - The Serenity context used for accessing Discord API and application data.
//! * `command_interaction` - The interaction object representing the command invocation by the user.
//!
use crate::command::command::Command;
use crate::command::embed_content::EmbedsContents;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::impl_command;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
    Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
    MediaQuerrySearchVariables, MediaType,
};
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

#[derive(Clone)]
pub struct LnCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for LnCommand,
	get_contents = |self_: LnCommand| async move {
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();

		let anilist_cache = bot_data.anilist_cache.clone();
		let _config = bot_data.config.clone();
		let map = get_option_map_string(&command_interaction);

		let value = map
			.get(&FixedString::from_str_trunc("ln_name"))
			.cloned()
			.unwrap_or(String::new());

		let data: Media = if value.parse::<i32>().is_ok() {
			let id = value.parse::<i32>()?;

			let var = MediaQuerryIdVariables {
				format_in: Some(vec![Some(MediaFormat::Novel)]),
				id: Some(id),
				media_type: Some(MediaType::Manga),
			};

			let operation = MediaQuerryId::build(var);

			let data: GraphQlResponse<MediaQuerryId> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().media.unwrap()
		} else {
			let var = MediaQuerrySearchVariables {
				format_in: Some(vec![Some(MediaFormat::Novel)]),
				search: Some(&*value),
				media_type: Some(MediaType::Manga),
			};

			let operation = MediaQuerrySearch::build(var);

			let data: GraphQlResponse<MediaQuerrySearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().media.unwrap()
		};
		let db_connection = bot_data.db_connection.clone();

		let embed_content =
			media::media_content(ctx, command_interaction, data, db_connection, bot_data).await?;

		Ok(embed_content)
	}
);
