use cynic::{GraphQlResponse, QueryBuilder};
use rand::{Rng, rng};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::background_task::update_random_stats::update_random_stats;
use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::random::load_localization_random;
use crate::structure::run::anilist::random::{
	MediaType, RandomPageMedia, RandomPageMediaVariables,
};
use anyhow::{Result, anyhow};

pub struct RandomCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RandomCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for RandomCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized random strings
		let random_localised = load_localization_random(guild_id, config.db.clone()).await?;

		// Retrieve the type of media (anime or manga) from the command interaction
		let map = get_option_map_string(command_interaction);

		let random_type = map
			.get(&FixedString::from_str_trunc("type"))
			.ok_or(anyhow!("No type specified"))?;

		self.defer().await?;

		let random_stats = update_random_stats(anilist_cache.clone()).await?;

		let last_page = if random_type.as_str() == "anime" {
			random_stats.anime_last_page
		} else if random_type.as_str() == "manga" {
			random_stats.manga_last_page
		} else {
			0
		};

		trace!(last_page);

		let number = rng().random_range(1..=last_page);

		let mut var = RandomPageMediaVariables {
			media_type: None,
			page: Some(number),
		};

		if random_type == "manga" {
			var.media_type = Some(MediaType::Manga)
		} else {
			var.media_type = Some(MediaType::Anime);
		}

		let operation = RandomPageMedia::build(var);

		let data: Result<GraphQlResponse<RandomPageMedia>> =
			make_request_anilist(operation, false, anilist_cache).await;

		let data = data?;

		let data = data.data.unwrap();

		let inside_media = data.page.unwrap().media.unwrap()[0].clone().unwrap();

		let id = inside_media.id;

		let url = if random_type == "manga" {
			format!("https://anilist.co/manga/{}", id)
		} else {
			format!("https://anilist.co/anime/{}", id)
		};

		let media = inside_media;

		let format = media.format.unwrap();

		let genres = media
			.genres
			.unwrap()
			.into_iter()
			.map(|genre| genre.unwrap().clone())
			.collect::<Vec<String>>()
			.join("/");

		let tags = media
			.tags
			.unwrap()
			.into_iter()
			.map(|tag| tag.unwrap().name.clone())
			.collect::<Vec<String>>()
			.join("/");

		let mut desc = media.description.unwrap();

		desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);

		let length_diff = 4096 - desc.len() as i32;

		if length_diff <= 0 {
			desc = trim(desc.clone(), length_diff);
		}

		let title = media.title.clone().unwrap();

		let rj = title.native.unwrap_or_default();

		let user_pref = title.user_preferred.unwrap_or_default();

		let title = format!("{}/{}", user_pref, rj);

		let full_desc = random_localised
			.desc
			.replace("$format$", format.to_string().as_str())
			.replace("$tags$", tags.as_str())
			.replace("$genres$", genres.as_str())
			.replace("$desc$", desc.as_str());

		let content = EmbedContent::new(title)
			.description(full_desc)
			.url(Some(url))
			.command_type(EmbedType::Followup);

		self.send_embed(vec![content]).await
	}
}
