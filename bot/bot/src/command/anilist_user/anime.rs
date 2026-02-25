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
use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
	Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
	MediaQuerrySearchVariables, MediaType,
};
use anyhow::anyhow;
use cynic::{GraphQlResponse, QueryBuilder};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

#[slash_command(
	name = "anime", desc = "Info of an anime.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "anime_name", desc = "Name of the anime you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn anime_command(self_: AnimeCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(self_.get_ctx().clone(), self_.get_command_interaction().clone());

	let map = get_option_map_string(&cx.command_interaction);
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

	let anilist_cache = cx.anilist_cache.clone();

	let data: Media = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>().unwrap();

		let var = MediaQuerryIdVariables {
			format_in,
			id: Some(id),
			media_type: Some(MediaType::Anime),
		};

		let operation = MediaQuerryId::build(var);

		let data: GraphQlResponse<MediaQuerryId> =
			make_request_anilist(operation, true, anilist_cache).await?;

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
			make_request_anilist(operation, true, anilist_cache).await?;

		match data.data {
			Some(data) => match data.media {
				Some(media) => media,
				None => return Err(anyhow!("Anime not found")),
			},
			None => return Err(anyhow!("Anime not found")),
		}
	};
	let embed_contents =
		media::media_content(cx.ctx, cx.command_interaction, data, cx.db, cx.bot_data).await?;

	Ok(embed_contents)
}
