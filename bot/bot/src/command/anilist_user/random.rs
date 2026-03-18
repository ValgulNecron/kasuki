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
use anyhow::Context;
use cynic::{GraphQlResponse, QueryBuilder};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use rand::{rng, RngExt};
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::trace;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::trimer::trim;
use crate::structure::run::anilist::random::{
	MediaType, RandomPageMedia, RandomPageMediaVariables,
};
use anyhow::{anyhow, Result};
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

	// Get the language identifier for localization
	let lang_id = cx.lang_id().await;

	// Retrieve the type of media (anime or manga) from the command interaction
	let map = get_option_map_string(&cx.command_interaction);

	let random_type = map
		.get(&FixedString::from_str_trunc("type"))
		.ok_or(anyhow!("No type specified"))?;

	let stats = shared::database::random_stats::Entity::find_by_id(1)
		.one(&*cx.db)
		.await
		.context("Failed to query random stats from database")?
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

	if random_type == "manga" {
		var.media_type = Some(MediaType::Manga)
	} else {
		var.media_type = Some(MediaType::Anime);
	}

	let operation = RandomPageMedia::build(var);

	let data: Result<GraphQlResponse<RandomPageMedia>> =
		make_request_anilist(operation, true, anilist_cache).await;

	let data = data?;

	let data = data.data.ok_or(anyhow!("No data found"))?;

	let inside_media = data
		.page
		.ok_or(anyhow!("No page found"))?
		.media
		.ok_or(anyhow!("No media found"))?
		.get(0)
		.cloned()
		.ok_or(anyhow!("No media found"))?
		.ok_or(anyhow!("No media found"))?;

	let id = inside_media.id;

	let url = if random_type == "manga" {
		format!("https://anilist.co/manga/{}", id)
	} else {
		format!("https://anilist.co/anime/{}", id)
	};

	let media = inside_media;

	let format = media.format.ok_or(anyhow!("No format found"))?;

	let genres = media
		.genres
		.ok_or(anyhow!("No genres found"))?
		.into_iter()
		.map(|genre| genre.unwrap_or_default())
		.collect::<Vec<String>>()
		.join("/");

	let tags = media
		.tags
		.ok_or(anyhow!("No tags found"))?
		.into_iter()
		.map(|tag| tag.unwrap().name.clone())
		.collect::<Vec<String>>()
		.join("/");

	let mut desc = media.description.ok_or(anyhow!("No description found"))?;

	desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);

	let length_diff = 4096 - desc.len() as i32;

	if length_diff <= 0 {
		desc = trim(desc.clone(), length_diff);
	}

	let title = media.title.clone().ok_or(anyhow!("No title found"))?;

	let rj = title.native.unwrap_or_default();

	let user_pref = title.user_preferred.unwrap_or_default();

	let title = format!("{}/{}", user_pref, rj);

	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("format"),
		FluentValue::from(format.to_string()),
	);
	args.insert(Cow::Borrowed("tags"), FluentValue::from(tags.as_str()));
	args.insert(Cow::Borrowed("genres"), FluentValue::from(genres.as_str()));
	args.insert(Cow::Borrowed("desc"), FluentValue::from(desc.as_str()));

	let full_desc = USABLE_LOCALES.lookup_with_args(&lang_id, "anilist_user_random-desc", &args);

	let embed_content = EmbedContent::new(title).description(full_desc).url(url);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
