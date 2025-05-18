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
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
	MediaQuerrySearchVariables, MediaType,
};
use anyhow::Result;
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

/// Represents a Lightning Network (LN) command within a bot's context, providing
/// the necessary context and interaction details for handling the command.
///
/// The `LnCommand` struct encapsulates data required to process an LN-related command,
/// including the bot's execution context and details of the user's interaction.
///
/// # Fields
/// - `ctx`:
///   The `SerenityContext` provides access to the Discord bot's runtime environment.
///   It contains information required to interact with the Discord API, such as
///   managing events, sending messages, and accessing shared data.
///   This is essential for executing and responding to the command within Discord.
///
/// - `command_interaction`:
///   The `CommandInteraction` represents the user's interaction with the bot.
///   It holds data specific to the command invocation, such as the command's name,
///   parameters provided by the user, and other metadata.
///   This is used to parse and respond to the command execution appropriately.
///
/// # Usage
/// This struct is designed to be used in scenarios where a bot processes a Lightning Network-related
/// command in Discord. It allows developers to efficiently manage the interaction
/// and context required during the command lifecycle.
pub struct LnCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LnCommand {
	/// Retrieves a reference to the Serenity context.
	///
	/// # Returns
	/// A reference to the `SerenityContext` stored within the current instance.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the returned `context` for further operations.
	/// ```
	///
	/// This method is immutable and does not modify the state of the instance.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object (`&CommandInteraction`) stored within the instance.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// An asynchronous function that retrieves a list of embed content for a media item from AniList
	/// based on the command interaction provided. This supports both media ID and name-based lookups.
	///
	/// # Returns
	/// A `Result` containing:
	/// - `Vec<EmbedContent<'_, '_>>`: A vector of embed content for the media item.
	/// - `Error`: If an error occurs during the process.
	///
	/// # Process
	/// 1. Retrieves the context and bot data.
	/// 2. Extracts the user's command interaction input (potentially an ID or name of a media item).
	/// 3. Checks if the user's input can be parsed as an `i32` (interpreted as a media ID):
	///     - If it is an ID, constructs a query to fetch media details using the media ID.
	///     - If it is not an ID, constructs a query to search for media using its name.
	/// 4. Sends a GraphQL request to the AniList API for the appropriate query based on the input.
	/// 5. Processes the API response to extract relevant media data.
	/// 6. Generates embed content using the `media_content` utility function with the media data,
	///    configuration, and bot data.
	/// 7. Returns the generated embed content.
	///
	/// # Arguments
	/// - `&self`: A reference to the current instance of the object implementing this method.
	///
	/// # Dependencies
	/// - `self.get_ctx()`: Retrieves the current execution context.
	/// - `self.get_command_interaction()`: Retrieves the command interaction details.
	/// - `get_option_map_string`: Utility that extracts options from the command interaction.
	/// - `make_request_anilist`: Sends and handles a GraphQL request to the AniList API.
	/// - `media::media_content`: Constructs the embed content from fetched media data.
	///
	/// # Errors
	/// This function returns an error if:
	/// - The user's input cannot be parsed as a valid ID.
	/// - The AniList API request fails or returns an invalid response.
	/// - Media data is not found for the provided ID or name.
	/// - Errors occur while generating embed content.
	///
	/// # Example Usage
	/// ```rust
	/// let embed_contents = my_instance.get_contents().await?;
	/// for embed_content in embed_contents {
	///     // Process your embed contents (e.g., send a response to Discord, log results, etc.)
	/// }
	/// ```
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();
		let map = get_option_map_string(command_interaction);

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

		let embed_content =
			media::media_content(ctx, command_interaction, data, config.db.clone(), bot_data)
				.await?;

		Ok(embed_content)
	}
}
