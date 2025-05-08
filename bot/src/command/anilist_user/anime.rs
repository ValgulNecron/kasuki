//! The `AnimeCommand` struct and its implementation allow users to retrieve anime-related metadata
//! from the AniList API via a Discord bot interaction. This struct uses the `Command` trait.
//!
//! # Struct Fields
//! - `ctx`: The serenity context (`SerenityContext`) providing access to Discord resources and the Discord API.
//! - `command_interaction`: The interaction (`CommandInteraction`) triggered by the user, containing information such as user input and context.
//!
//! # Trait Implementation: `Command`
//!
//! Provides functionality to use this struct within the bot's command system.
//!
//! ### Functions
//!
//! #### `get_ctx`
//! - Returns a reference to the serenity context (`SerenityContext`).
//! - **Usage**: Used internally to access bot-related resources.
//!
//! #### `get_command_interaction`
//! - Returns a reference to the `CommandInteraction`.
//! - **Usage**: Allows access to user interaction data, including command options and user context.
//!
//! #### `get_contents`
//! - **Returns**: A `Result` containing a vector of `EmbedContent` if successful, or an error (`anyhow::Error`) if not.
//! - **Async**: This function is asynchronous and must be awaited.
//! - Retrieves metadata for an anime based on the user's input.
//! - Utilizes the AniList API to perform anime queries:
//!   - If the input is a numeric ID, the lookup is based on the AniList ID.
//!   - If the input is a string, the API performs a search using the provided string as the search term.
//! - Handles media formats such as TV, TV shorts, movies, specials, OVAs, ONAs, and music.
//! - Builds dynamic GraphQL queries based on user input and executes those queries using caching mechanisms for optimization.
//! - If an anime is found:
//!   - Uses `media_content` to generate an `EmbedContent` to be displayed in the user's Discord client.
//! - If no anime is found, returns an appropriate error.
//!
//! # Dependencies
//!
//! - `crate::command::command_trait::{Command, CommandRun, EmbedContent}`: Provides the necessary traits for defining commands and their behaviors.
//! - `crate::helper::get_option::command::get_option_map_string`: Helper to fetch command options from the interaction.
//! - `crate::helper::make_graphql_cached::make_request_anilist`: Handles API requests to AniList with cache optimization.
//! - `serenity::all::{CommandInteraction, Context as SerenityContext}`: Provides the context and interaction models needed for serenity bot interactions.
//! - `cynic::{GraphQlResponse, QueryBuilder}`: Used for constructing and handling GraphQL query responses.
//! - `anyhow`: For error handling.
//! - `small_fixed_array::FixedString`: Helps handle fixed string operations when extracting user-provided options.
//!
//! # Error Handling
//!
//! - Returns an error when:
//!   - No anime data is found for the given input.
//!   - Issues arise with the AniList API or GraphQL query execution.
//!
//! # Example
//!
//! ```rust
//! let anime_command = AnimeCommand {
//!     ctx: serenity_context_instance,
//!     command_interaction: user_interaction_instance,
//! };
//!
//! let result = anime_command.get_contents().await;
//! match result {
//!     Ok(embed_contents) => {
//!         // Process and display the returned anime details as an embed.
//!     },
//!     Err(error) => {
//!         // Handle the error (e.g., log it or notify the user that the anime was not found).
//!     },
//! }
//! ```
use crate::command::command_trait::{Command, CommandRun, EmbedContent};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
	MediaQuerrySearchVariables, MediaType,
};
use anyhow::{Result, anyhow};
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

/// This struct represents a command related to Anime in the context
/// of a Discord bot using the `serenity` library. It holds the necessary
/// context and interaction data required for handling the command.
///
/// # Fields
///
/// - `ctx` - The `SerenityContext`, which provides access to the Discord bot's
///           client state, allowing the command to perform various operations
///           in the Discord API, such as sending messages or managing guilds.
///
/// - `command_interaction` - The `CommandInteraction`, which contains 
///                           information about the specific command interaction.
///                           This includes the user who invoked the command,
///                           the guild or channel it was invoked in, as well as
///                           the command data itself.
pub struct AnimeCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AnimeCommand {
	/// Retrieves a reference to the stored `SerenityContext`.
	///
	/// # Returns
	/// A reference to the `SerenityContext` instance associated with the current object.
	///
	/// # Examples
	/// ```rust
	/// let context = my_object.get_ctx();
	/// // Use `context` for further operations
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the stored `CommandInteraction`.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` associated with the instance.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// // Use `command_interaction` as needed
	/// ```
	///
	/// This method is useful when you need to access the `CommandInteraction`
	/// without taking ownership of it.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves a list of embed contents based on user input, such as anime name or ID.
	///
	/// # Errors
	///
	/// Returns an error if:
	/// - The anime is not found in the API response.
	/// - There is an issue with the API request or response parsing.
	///
	/// # Returns
	///
	/// Returns a `Result` containing a vector of `EmbedContent` if successful, or an error on failure.
	///
	/// # Details
	///
	/// This function performs the following operations:
	/// 1. Retrieves the context and bot data needed for the operation.
	/// 2. Extracts and processes the anime name or ID from the command interaction's options.
	/// 3. Prepares a set of media formats (`format_in`) for filtering anime media types.
	/// 4. Executes a query against the AniList API to fetch media data:
	///    - If the input value is an integer, the function treats it as an anime ID and queries based on ID.
	///    - If the input value is a string, the function treats it as an anime name and performs a search query.
	/// 5. Parses the API's GraphQL response to extract the relevant media object.
	/// 6. Generates embed content using the fetched media data and additional bot configuration.
	///
	/// # Example
	///
	/// ```rust
	/// let embed_contents = my_obj.get_contents().await?;
	/// for content in embed_contents {
	///     println!("{:?}", content);
	/// }
	/// ```
	///
	/// This function relies on types and modules such as `BotData`, `MediaFormat`, `MediaType`, `GraphQlResponse`,
	/// `MediaQuerryId`, `MediaQuerrySearch`, and various utility functions (e.g., `make_request_anilist`, `media_content`).
	///
	/// Ensure that all relevant dependencies are properly defined and that the context contains the necessary data for execution.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
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

		let embed_content =
			media::media_content(ctx, command_interaction, data, config.db.clone(), bot_data)
				.await?;
		
		Ok(embed_content)
	}
}
