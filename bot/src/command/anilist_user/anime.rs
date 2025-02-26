use std::sync::Arc;

use crate::command::command_trait::{Command, Embed, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
	MediaQuerrySearchVariables, MediaType,
};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct AnimeCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AnimeCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for AnimeCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let map = get_option_map_string(command_interaction);

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

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();

		let data: Media = if value.parse::<i32>().is_ok() {
			let id = value.parse::<i32>().unwrap();

			let var = MediaQuerryIdVariables {
				format_in,
				id: Some(id),
				media_type: Some(MediaType::Anime),
			};

			let operation = MediaQuerryId::build(var);

			let data: GraphQlResponse<MediaQuerryId> =
				make_request_anilist(operation, false, anilist_cache).await?;

			match data.data {
				Some(data) => match data.media {
					Some(media) => media,
					None => return Err(anyhow!("Anime not found")),
				},
				None => return Err(anyhow!("Anime not found")),
			}
		} else {
			let var = MediaQuerrySearchVariables {
				format_in,
				search: Some(&*value),
				media_type: Some(MediaType::Anime),
			};

			let operation = MediaQuerrySearch::build(var);

			let data: GraphQlResponse<MediaQuerrySearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			match data.data {
				Some(data) => match data.media {
					Some(media) => media,
					None => return Err(anyhow!("Anime not found")),
				},
				None => return Err(anyhow!("Anime not found")),
			}
		};

		let content = media::media_content(ctx, command_interaction, data, config.db.clone()).await?;

		self.send_embed(content).await
	}
}
