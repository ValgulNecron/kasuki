//! The `CharacterCommand` struct represents a command for working with anime or manga
//! characters from the AniList API. It handles fetching character data based on a user
//! provided name or ID and constructs appropriate embedded content.
//!
//! # Fields
//! * `ctx` - The `SerenityContext` instance for interacting with Discord's API and gateway.
//! * `command_interaction` - The interaction data from the Discord command input.
//!
//! The implementation of the `CharacterCommand` provides the following functionalities:
//! - Fetching execution context via `get_ctx`
//! - Retrieving the Discord command interaction via `get_command_interaction`
//! - Fetching content to display as the command result via `get_contents`
use std::sync::Arc;

use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character;
use crate::structure::run::anilist::character::{
	Character, CharacterQuerryId, CharacterQuerryIdVariables, CharacterQuerrySearch,
	CharacterQuerrySearchVariables,
};
use anyhow::{anyhow, Context, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::cache::CacheInterface;
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

#[slash_command(
	name = "character", desc = "Info of a character.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the character you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn character_command(self_: CharacterCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);
	let value = map
		.get(&FixedString::from_str_trunc("name"))
		.cloned()
		.unwrap_or(String::new());

	let data: Character = if value.parse::<i32>().is_ok() {
		let character_id = value
			.parse::<i32>()
			.context("Failed to parse character ID as integer")?;
		get_character_by_id(character_id, anilist_cache)
			.await
			.context(format!("Failed to get character with ID {}", character_id))?
	} else {
		let var = CharacterQuerrySearchVariables {
			search: Some(&*value),
		};

		let operation = CharacterQuerrySearch::build(var);

		let data: GraphQlResponse<CharacterQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache)
				.await
				.context(format!(
					"Failed to make AniList API request for character search with query '{}'",
					value
				))?;

		data.data
			.context("No data returned from AniList API for character search")?
			.character
			.context(format!("No character found with name '{}'", value))?
	};
	let embed_contents = character::character_content(cx.command_interaction, data, cx.db)
		.await
		.context("Failed to generate character content for embed")?;

	Ok(embed_contents)
}

/// Retrieves a specific character from the AniList API based on the provided character ID.
///
/// # Arguments
///
/// * `value` - The ID of the character as an `i32` that needs to be fetched.
/// * `anilist_cache` - A shared and thread-safe reference to a cache object wrapped in an `Arc<RwLock<Cache<String, String>>>`.
///
/// The function constructs a GraphQL query with the provided character ID, sends the request to the AniList API,
/// and fetches the relevant character information. The cache object is utilized to optimize and reduce the number
/// of direct requests to the API by caching results.
///
/// The retrieved response is parsed and validated to ensure that the character data is available and acceptable.
/// If no character data or no API data is returned, an error is propagated.
///
/// # Returns
///
/// Upon success, returns a `Result<Character>` where `Character` contains all the fetched character details.
/// In case of an error during request execution or if the response does not contain the necessary data,
/// an `Err(anyhow)` is returned.
///
/// # Errors
///
/// This function returns an error in the following scenarios:
/// * No data is received from the API.
/// * A character with the given ID does not exist in the AniList database.
/// * Any failures during the request process, such as connection issues.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use your_crate_name::get_character_by_id;
/// use your_cache_library::Cache;
///
/// #[tokio::main]
/// async fn main() {
///     let cache = Arc::new(RwLock::new(Cache::new()));
///     let character_id = 12345;
///
///     match get_character_by_id(character_id, cache).await {
///         Ok(character) => println!("Character fetched: {:?}", character),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
///
/// # Note
///
/// Make sure the AniList API integration and cache handling is properly configured for this function to work as expected.
/// The `Cache` and `GraphQlResponse` types should be pre-defined and available in your crate or the imported modules.
pub async fn get_character_by_id(
	value: i32, anilist_cache: Arc<RwLock<CacheInterface>>,
) -> Result<Character> {
	let var = CharacterQuerryIdVariables { id: Some(value) };

	let operation = CharacterQuerryId::build(var);

	let data: GraphQlResponse<CharacterQuerryId> =
		make_request_anilist(operation, true, anilist_cache)
			.await
			.context(format!(
				"Failed to make AniList API request for character with ID {}",
				value
			))?;

	match data.data {
		Some(data) => match data.character {
			Some(media) => Ok(media),
			None => Err(anyhow!("No character found with ID {}", value)
				.context("The character ID may not exist or may have been removed from AniList")),
		},
		None => Err(anyhow!(
			"No data returned from AniList API for character with ID {}",
			value
		)
		.context("This could indicate an issue with the AniList API or the request format")),
	}
}
