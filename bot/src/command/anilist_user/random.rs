//! The `RandomCommand` structure implements the `Command` trait to handle the logic for generating
//! and responding to a random anime or manga media request in a Discord bot.
//!
//! # Fields
//! - `ctx`: Represents the context of the Serenity Discord bot, providing access to shared data and interactions.
//! - `command_interaction`: Represents the user's interaction (slash command) with the bot.
//!
//! # Implementations
//! - `get_ctx`: Retrieves the current `SerenityContext` for the command.
//! - `get_command_interaction`: Retrieves the current interaction details for the command.
//! - `get_contents`: Asynchronously processes the random command to fetch a random anime or manga from AniList
//!   and returns the appropriate response in the form of an `EmbedContent`.
//!
//! # `get_contents` Method Details
//! The `get_contents` method performs the following tasks:
//! 1. Loads localized strings for responses based on the user's guild.
//! 2. Extracts the media type (anime or manga) from user's command options.
//! 3. Defers the response for longer processing times.
//! 4. Updates random statistics for the cache object.
//! 5. Determines the last page count for anime or manga based on the statistics.
//! 6. Randomly selects a page to retrieve data from AniList using a GraphQL query.
//! 7. Processes and Formats the retrieved media data, such as title, description, genres, and tags.
//! 8. Converts AniList-flavored Markdown to Discord-friendly Markdown.
//! 9. Ensures description length fits Discord's character limit by trimming if necessary.
//! 10. Formats and returns an embed response with the media details.
//!
//! # Notes
//! - AniList's GraphQL API is used for fetching randomized anime or manga data.
//! - The function makes use of localization for custom responses and formatting.
//! - Errors are propagated using `anyhow` to ensure any issues during execution are handled properly.
//!
//! # Example Return
//! - Embed response is generated with the following fields:
//!   - Title (native & user-preferred)
//!   - Description (with genres, tags, and additional details)
//!   - URL linking to the specific anime or manga on AniList.
//!
//! # Errors
//! - Returns an error if:
//!   - No type is specified in the command options.
//!   - Localization fails to load.
//!   - AniList GraphQL query fails.
//!   - Page data from AniList API is invalid or missing.
//!
//! # Dependencies
//! - `Serenity`: For Discord bot interactions.
//! - `cynic`: For GraphQL queries and responses.
//! - `rand`: For generating random page numbers.
//! - `tracing`: For logging debug and trace information.
//! - Custom modules for localization, caching, trimming, and AniList Markdown conversion.
use cynic::{GraphQlResponse, QueryBuilder};
use rand::{rng, Rng};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::background_task::update_random_stats::update_random_stats;
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim;
use crate::impl_command;
use crate::structure::message::anilist_user::random::load_localization_random;
use crate::structure::run::anilist::random::{
    MediaType, RandomPageMedia, RandomPageMediaVariables,
};
use anyhow::{anyhow, Result};

/// A `RandomCommand` struct that encapsulates the context and interaction details for a command
/// in a Discord bot using the Serenity library.
///
/// This struct is designed to handle a specific command interaction by bundling the necessary
/// context (`SerenityContext`) and the interaction details (`CommandInteraction`) into a single
/// entity.
///
/// # Fields
/// - `ctx`: The context of the current Discord bot session and environment. This provides
///   access to the bot's data, such as cache, HTTP client, and other utilities for responding
///   to events or commands.
/// - `command_interaction`: The interaction data for the specific command. This contains
///   information about the command invocation, including the user's input, options, and other
///   metadata required to handle and respond to the command.
///
/// # Example
/// ```
/// use serenity::prelude::*;
/// use serenity::model::interactions::application_command::CommandInteraction;
///
/// struct RandomCommand {
///     ctx: SerenityContext,
///     command_interaction: CommandInteraction,
/// }
///
/// // Example usage of RandomCommand
/// let random_command = RandomCommand {
///     ctx: some_serenity_context,
///     command_interaction: some_command_interaction,
/// };
///
/// // Use `random_command` to process the command interaction and respond accordingly.
/// ```
#[derive(Clone)]
pub struct RandomCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for RandomCommand,
	get_contents = |self_: RandomCommand| async move {
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.read().await.get_cache();
		let _config = bot_data.config.clone();
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let db_connection = bot_data.db_connection.clone();

		// Load the localized random strings
		let random_localised = load_localization_random(guild_id, db_connection).await?;

		// Retrieve the type of media (anime or manga) from the command interaction
		let map = get_option_map_string(command_interaction);

		let random_type = map
			.get(&FixedString::from_str_trunc("type"))
			.ok_or(anyhow!("No type specified"))?;

		self_.defer().await?;

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

		let embed_content = EmbedContent::new(title).description(full_desc).url(url);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);
