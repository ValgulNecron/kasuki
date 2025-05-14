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

use crate::command::command::{Command, CommandRun, EmbedContent};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character;
use crate::structure::run::anilist::character::{
	Character, CharacterQuerryId, CharacterQuerryIdVariables, CharacterQuerrySearch,
	CharacterQuerrySearchVariables,
};
use anyhow::{Result, anyhow};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

/// A struct representing a command that interacts with a user in the context of a Discord bot.
///
/// The `CharacterCommand` struct is used to encapsulate the context and the command interaction
/// needed to execute a command within a Discord bot, using the `serenity` library.
///
/// # Fields
///
/// * `ctx` - The context of the bot, represented by `SerenityContext`.
///   This includes access to bot state, configurations, and allows interaction with the Discord API.
///
/// * `command_interaction` - Represents the interaction payload, which contains all the information
///   and arguments provided when a user invokes a command.
///
/// This struct is typically used to process commands and respond to interactions in a Discord guild or direct message.
///
/// # Example
///
/// ```rust
/// let command = CharacterCommand {
///     ctx,
///     command_interaction,
/// };
///
/// // Use command.ctx and command.command_interaction in your logic
/// ```
pub struct CharacterCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for CharacterCommand {
	/// Returns a reference to the `SerenityContext` associated with the current object.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` stored within the object.
	///
	/// # Example
	/// ```rust
	/// let context = my_object.get_ctx();
	/// // Use the `context` as needed
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `command_interaction` field.
	///
	/// This method provides access to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```
	/// let interaction = instance.get_command_interaction();
	/// // Now `interaction` holds a reference to the CommandInteraction.
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously fetches character information from AniList and returns it as embed content.
	///
	/// # Arguments
	/// * `self` - Reference to the instance of the struct holding the required context and state.
	///
	/// # Returns
	/// A `Result` containing:
	/// * `Vec<EmbedContent<'_, '_>>`: A vector of embed content representing the character information.
	/// * An `Error` if any issue arises during the fetch or processing of the request.
	///
	/// # Workflow
	/// 1. Retrieves the execution context using `get_ctx`.
	/// 2. Fetches shared bot data and configuration.
	/// 3. Extracts the option map from the command interaction.
	/// 4. Parses the "name" field from the provided option map to determine the character query.
	///    - If the "name" is an integer, it is treated as a character ID.
	///    - Otherwise, it is treated as a character name for search.
	/// 5. If an ID is provided:
	///    - Fetches the character data using `get_character_by_id`.
	/// 6. If a name is provided:
	///    - Constructs a GraphQL query using the `CharacterQuerrySearchVariables` struct.
	///    - Sends the GraphQL request to AniList using `make_request_anilist`.
	/// 7. Extracts the character information from the API response.
	/// 8. Converts the character data into embed-compatible content using `character::character_content`.
	/// 9. Returns the embed content as a result.
	///
	/// # Errors
	/// The function can return an error in the following scenarios:
	/// * The "name" field in the command interaction is missing or invalid.
	/// * Issues with the AniList API request or response (e.g., connectivity, invalid query).
	/// * Errors occurring while generating embed content.
	///
	/// # Dependencies
	/// This function relies on various external and internal modules/utilities:
	/// - `get_ctx`, `get_command_interaction`: To access the command execution context.
	/// - `BotData`, `anilist_cache`, `config`: To access bot-level configurations and caching mechanisms.
	/// - `character::character_content`: To generate embed content.
	/// - `make_request_anilist`: To handle AniList API interactions.
	/// - `GraphQlResponse`, `Character`, `CharacterQuerrySearch`, `CharacterQuerrySearchVariables`: To process GraphQL requests and responses.
	///
	/// # Examples
	/// ```rust
	/// match my_instance.get_contents().await {
	///     Ok(contents) => {
	///         for content in contents {
	///             println!("Embed Content: {:?}", content);
	///         }
	///     },
	///     Err(e) => eprintln!("Error fetching character content: {}", e),
	/// }
	/// ```
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();

		let map = get_option_map_string(command_interaction);
		let value = map
			.get(&FixedString::from_str_trunc("name"))
			.cloned()
			.unwrap_or(String::new());

		let data: Character = if value.parse::<i32>().is_ok() {
			get_character_by_id(value.parse::<i32>().unwrap(), anilist_cache).await?
		} else {
			let var = CharacterQuerrySearchVariables {
				search: Some(&*value),
			};

			let operation = CharacterQuerrySearch::build(var);

			let data: GraphQlResponse<CharacterQuerrySearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().character.unwrap()
		};

		let embed_content =
			character::character_content(command_interaction, data, config.db.clone()).await?;

		Ok(embed_content)
	}
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
	value: i32, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Character> {
	let var = CharacterQuerryIdVariables { id: Some(value) };

	let operation = CharacterQuerryId::build(var);

	let data: GraphQlResponse<CharacterQuerryId> =
		make_request_anilist(operation, false, anilist_cache).await?;

	Ok(match data.data {
		Some(data) => match data.character {
			Some(media) => media,
			None => return Err(anyhow!("No character found")),
		},
		None => return Err(anyhow!("No data found")),
	})
}
