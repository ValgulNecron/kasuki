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
use rand::{Rng, rng};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::background_task::update_random_stats::update_random_stats;
use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
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
pub struct RandomCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RandomCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) held within the struct.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` for further operations
	/// ```
	///
	/// This function is typically used to access the context required for interacting with
	/// Discord's API or handling bot-related operations.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Returns a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` stored within the instance.
	///
	/// # Example
	///
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use the interaction reference for further operations
	/// ```
	///
	/// # Notes
	///
	/// This method provides read-only access to the `CommandInteraction`
	/// and does not modify the instance.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves embed contents based on media type and context.
	///
	/// This function is designed to fetch a random media (anime or manga) from the AniList API,
	/// construct detailed embed content with metadata, and return it for use in command interactions.
	///
	/// # Parameters
	/// This is an instance method, so it relies on the context/state contained within `self`.
	///
	/// # Returns
	/// - `Result<Vec<EmbedContent<'_, '_>>>`: A result containing a vector of embed content on success,
	/// or an error if something went wrong during the execution.
	///
	/// # Workflow
	/// 1. Initializes context and extracts cached data and command interaction.
	/// 2. Determines the media type (anime or manga) from the user's command interaction input.
	/// 3. Fetches random localized text for descriptions.
	/// 4. Retrieves and updates statistics about the available media pool from the AniList cache.
	/// 5. Calculates a random page number based on media type and fetches random media data from AniList.
	/// 6. Extracts and formats metadata such as title, description, genres, and tags.
	/// 7. Constructs a user-friendly embed content object with the fetched data.
	///
	/// # Details
	/// - **Localization**: Incorporates localized descriptions for randomness.
	/// - **API Integration**: Executes GraphQL queries against the AniList API to retrieve media information.
	/// - **Error Handling**: Handles various potential failures, including missing data, AniList API errors, or unfulfilled user input.
	///
	/// # Media Data
	/// The metadata fields included in the embed content:
	/// - **Title**: A combined format of the user-preferred title and native title.
	/// - **Description**: An abridged description of the media, formatted in Discord-flavored markdown.
	/// - **Genres**: Relevant genres of the media.
	/// - **Tags**: Detailed tags for further classification of the media.
	/// - **Media Format**: Specifies whether the media is a TV series, movie, manga, etc.
	/// - **URL**: Link to the AniList page of the selected media.
	///
	/// # Example
	/// ```rust
	/// let embed_contents = my_instance.get_contents().await;
	/// match embed_contents {
	///     Ok(embeds) => {
	///         for embed in embeds {
	///             // Process or display the embed
	///         }
	///     }
	///     Err(error) => {
	///         // Handle error
	///     }
	/// }
	/// ```
	///
	/// # Errors
	/// This function can return an error in the following cases:
	/// - If the media type is not specified in the command interaction.
	/// - If any AniList API request fails or returns invalid/missing data.
	/// - If there's an issue with localization, configuration, or cache retrieval.
	///
	/// # Dependencies
	/// - `ctx`: Contains the context and data related to bot operations.
	/// - `BotData`: Holds the AniList cache and configuration required to perform API requests and localization.
	/// - `make_request_anilist`: Executes AniList GraphQL queries.
	/// - `convert_anilist_flavored_to_discord_flavored_markdown`: Converts descriptions to be compatible with Discord markdown styling.
	/// - `EmbedContent`: A custom structure to build and format the Discord embed messages.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
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

		let embed_content = EmbedContent::new(title)
			.description(full_desc)
			.url(Some(url))
			.command_type(EmbedType::Followup);

		Ok(vec![embed_content])
	}
}
