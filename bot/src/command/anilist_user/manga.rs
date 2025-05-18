//! This module defines the `MangaCommand` structure and its implementation for fetching manga
//! information using the AniList GraphQL API. It forms part of a bot system based on the Serenity library
//! for managing commands and interactions.
use crate::command::command::Command;
use crate::command::embed_content::EmbedsContents;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
	MediaQuerrySearchVariables, MediaType,
};
use anyhow::Result;
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

/// A structure representing a command related to manga, intended for handling interactions within a Discord bot.
///
/// # Fields
///
/// * `ctx` - The context associated with the Serenity framework,
///           providing access to resources and services for handling the bot's operation.
/// * `command_interaction` - The interaction data received from a user when they trigger the command,
///                           containing information such as arguments and metadata.
///
/// This structure is typically utilized in scenarios where a manga-related command is executed in a Discord server,
/// allowing access to both the context and interaction details for processing the command effectively.
pub struct MangaCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for MangaCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`).
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Now you can use `context` to perform operations that require the Serenity context.
	/// ```
	///
	/// This method is typically used to interface with the Discord bot's context,
	/// enabling access to bot state, data, and functionality provided by the Serenity framework.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Returns a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object (`&CommandInteraction`) that is
	/// stored in the instance of the structure.
	///
	/// # Example
	/// ```
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Fetches and returns the content for an embed related to a manga based on the user's input.
	///
	/// The function retrieves the bot's shared data, the command interaction, and the user's
	/// input parameters. It checks if the input value is a numerical ID or a string and then:
	/// - Fetches the manga data using the ID if the value is a valid integer.
	/// - Performs a search to fetch the manga data if the value is a string.
	///
	/// The retrieved data is used to generate embed content.
	///
	/// # Returns
	///
	/// Returns a `Result` containing:
	/// - `Ok`: A vector of `EmbedContent` objects if the operation is successful.
	/// - `Err`: An error in case of any issues during the data fetching or processing.
	///
	/// # Steps
	///
	/// 1. Retrieve the bot's shared context and configuration.
	/// 2. Extract the user's input for manga identification.
	/// 3. Determine whether the input is an ID or a textual search term:
	///    - If it is a valid integer ID, a GraphQL query by ID is performed.
	///    - If it is not a valid integer, a GraphQL search query is used with keywords.
	/// 4. Use the fetched `Media` data to generate the embed content for Discord.
	///
	/// # Errors
	///
	/// - Returns an error if the input parsing fails.
	/// - Returns an error if there are issues with the GraphQL requests or responses.
	/// - Returns an error if generating the embed content fails.
	///
	/// # Example
	///
	/// ```rust
	/// let embed_contents = handler.get_contents().await?;
	/// for content in embed_contents {
	///     // Do something with embed content
	/// }
	/// ```
	///
	/// # Dependencies
	///
	/// - `make_request_anilist`: Function to make requests to the AniList API.
	/// - `MediaQuerryId` and `MediaQuerrySearch`: GraphQL queries for fetching manga data.
	/// - `media::media_content`: Helper function to format media data for embed content.
	///
	/// # Notes
	///
	/// - Supports fetching 'Manga' and 'One-Shot' formats.
	/// - Caches data when possible to minimize external API requests.
	///
	/// # Related
	///
	/// - Uses `BotData` shared state for configurations and caches.
	/// - Depends on Discord command interaction to extract user input.
	///
	/// # Parameters
	///
	/// - `self`: Represents the instance containing the context to execute the function.
	/// - Returns a `Result` object with the success payload `Vec<EmbedContent<'_, '_>>`.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();
		let map = get_option_map_string(command_interaction);

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
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().media.unwrap()
		} else {
			let var = MediaQuerrySearchVariables {
				format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
				search: Some(&*value),
				media_type: Some(MediaType::Manga),
			};

			let operation = MediaQuerrySearch::build(var);

			let data: GraphQlResponse<MediaQuerrySearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().media.unwrap()
		};

		let embed_content =
			media::media_content(ctx, command_interaction, data, config.db.clone(), bot_data)
				.await?;

		Ok(embed_content)
	}
}
