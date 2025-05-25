//! Struct and implementation for handling the Studio command in a Discord bot using Serenity.
//!
//! The StudioCommand provides functionality to interact with AniList API, fetch information about a specific studio
//! (either by ID or search query), and format the response as an embed message for the Discord bot. The command interaction
//! makes use of cached requests to minimize API calls and provides localized content based on the user's guild settings.
//!
//! # Structs
//!
//! ## `StudioCommand`
//! This is the main struct representing the Studio command interaction. It is responsible for:
//! - Holding the Serenity context and command interaction details (`ctx` and `command_interaction`).
//! - Fetching input provided by the user through Discord slash command options.
//! - Processing data from the AniList API to create a well-structured embed response.
//!
//! ## Functions/Implementations
//!
//! ### `impl Command for StudioCommand`
//! This trait implementation provides the necessary methods to conform to the bot's command framework.
//!
//! #### `get_ctx`
//! Returns a reference to the Serenity context associated with the command.
//! - **Returns**: `&SerenityContext`
//!
//! #### `get_command_interaction`
//! Returns a reference to the command interaction associated with the command.
//! - **Returns**: `&CommandInteraction`
//!
//! #### `get_contents`
//! This is the primary command execution logic. It fetches the studio data based on the user input, processes the results,
//! and formats it into embed content for the response.
//! - **Returns**:
//!   - `Ok(Vec<EmbedContent<'_, '_>>)` on success, containing the formatted embed data.
//!   - `Err(anyhow::Error)` if any step in data fetching or processing fails.
//!
//! The workflow is as follows:
//! 1. Retrieve the command context and interaction details.
//! 2. Extract the studio query (either a number for ID or a search string for a name).
//! 3. Query AniList API for studio information by ID or search name, using cached responses where possible.
//! 4. Retrieve localization based on the guild's ID and database configuration.
//! 5. Process and append the media list linked to the studio as part of the content.
//! 6. Construct the embed description with the localized strings, studio metadata, and media list.
//! 7. Return embed content formatted with the studio details.
//!
//! ### Error Handling
//! The command gracefully handles and returns errors in the following scenarios:
//! - When no `studio` option is specified in the command interaction.
//! - When the AniList API call fails (e.g., bad requests, network issues, or parsing issues).
//! - When localization or guild-related data cannot be fetched.
//!
//! # Dependencies
//!
//! The implementation relies on the following crates and modules:
//! - `anyhow`: For concise error handling and propagation.
//! - `serenity`: For Discord bot framework interaction.
//! - `cynic`: For constructing GraphQL queries and parsing API responses.
//! - Custom modules for command traits, constants, helpers, and data structures.
//!
//! # Example
//!
//! An example of executing the StudioCommand:
//!
//! ```rust
//! let studio_cmd = StudioCommand {
//!     ctx, // Serenity context
//!     command_interaction, // Discord command interaction details
//! };
//! let result = studio_cmd.get_contents().await;
//!
//! match result {
//!     Ok(contents) => {
//!         // Send embed message using the `contents`
//!     },
//!     Err(err) => {
//!         // Log or handle the error
//!     }
//! }
//! ```
//!
//! In this example, the command fetches data for a studio either by ID or search query and formats
//! it into a detailed response, including media information, localization, and more.
use crate::command::command::Command;
use anyhow::{Result, anyhow};

use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::studio::load_localization_studio;
use crate::structure::run::anilist::studio::{
	StudioQuerryId, StudioQuerryIdVariables, StudioQuerrySearch, StudioQuerrySearchVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

/// Represents a command executed within the context of a studio application.
///
/// This struct encapsulates the necessary data to process and execute a command
/// within the context of the Serenity framework, enabling interaction with the
/// Discord API and user-issued commands.
///
/// # Fields
///
/// * `ctx` - The context of the Serenity framework, providing access to
///   Discord-related functionality and structures, such as guilds, channels,
///   and users.
/// * `command_interaction` - The interaction details related to the command
///   issued by a user, including the command name, arguments, and metadata.
///
/// # Usage
///
/// The `StudioCommand` struct is utilized to represent the union of the
/// Serenity context and the specific command interaction being processed.
/// It is commonly used in managing and responding to user commands.
///
/// # Example
///
/// ```rust
/// use my_crate::StudioCommand;
///
/// fn handle_command(command: StudioCommand) {
///     // Access the Serenity context using command.ctx
///     // Access the command interaction using command.command_interaction
/// }
/// ```
pub struct StudioCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for StudioCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` held by this instance.
	///
	/// # Examples
	/// ```
	/// let context = instance.get_ctx();
	/// // Use the context for further operations.
	/// ```
	///
	/// # Notes
	/// This method provides immutable access to the context.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// - A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```
	/// let interaction = instance.get_command_interaction();
	/// // Use the `interaction` object as needed.
	/// ```
	///
	/// This method provides read-only access to the `command_interaction` field of the struct it is implemented for.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously fetches and compiles the contents of a studio's information from AniList.
	///
	/// # Returns
	/// A `Result` containing either:
	/// - A `Vec<EmbedContent<'_, '_>>` when the operation is successful.
	/// - An error if any step in the process fails.
	///
	/// # Steps
	/// 1. Retrieves the bot context and fetches associated data, including configuration and AniList cache.
	/// 2. Parses the studio argument from the command interaction:
	///    - If the provided studio argument is a numeric ID, fetches the studio data by its ID.
	///    - Otherwise, treats the given value as a name and searches for the studio by name.
	/// 3. Handles localization to support dynamic response descriptions based on the guild ID.
	/// 4. Constructs a content string containing links and titles for each media associated with the studio.
	/// 5. Builds and returns an embed containing the studio's information.
	///
	/// # Errors
	/// Returns an error if any of the following occur:
	/// - The studio argument is not specified.
	/// - Parsing or retrieving data via AniList fails.
	/// - Localization or database-related operations fail.
	///
	/// # Example
	/// ```
	/// # async {
	/// let contents = some_command.get_contents().await;
	/// match contents {
	///     Ok(embed_contents) => {
	///         // Process and send the embed contents.
	///     }
	///     Err(err) => {
	///         // Handle errors, e.g., log or notify the user.
	///     }
	/// }
	/// # };
	/// ```
	///
	/// # Dependencies
	/// - `BotData`: For accessing the configuration and AniList cache.
	/// - GraphQL queries: Used for fetching data from AniList.
	/// - Localization utilities: Required to construct localized studio strings.
	///
	/// # Notes
	/// - The studio data is fetched via AniList's GraphQL API using either the studio's ID or name.
	/// - Embed descriptions are dynamically constructed using localization strings and studio metadata.
	/// - If there is no guild ID specified in the command interaction, a default guild ID of "0" is assigned.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let config = bot_data.config.clone();

		let anilist_cache = bot_data.anilist_cache.clone();

		let map = get_option_map_string(command_interaction);

		let value = map
			.get(&FixedString::from_str_trunc("studio"))
			.ok_or(anyhow!("No studio specified"))?;

		// Fetch the studio's data from AniList
		let studio = if value.parse::<i32>().is_ok() {
			let id = value.parse::<i32>()?;

			let var = StudioQuerryIdVariables { id: Some(id) };

			let operation = StudioQuerryId::build(var);

			let data: GraphQlResponse<StudioQuerryId> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().studio.unwrap()
		} else {
			let var = StudioQuerrySearchVariables {
				search: Some(value.as_str()),
			};

			let operation = StudioQuerrySearch::build(var);

			let data: GraphQlResponse<StudioQuerrySearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().studio.unwrap()
		};

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized studio strings
		let studio_localised = load_localization_studio(guild_id, config.db.clone()).await?;

		// Initialize a string to store the content of the response
		let mut content = String::new();

		// Iterate over the nodes of the studio's media
		for m in studio.media.unwrap().nodes.unwrap() {
			// Clone the title of the media
			let m = m.unwrap();

			let title = m.title.unwrap().clone();

			// Retrieve the romaji and user-preferred titles
			let rj = title.romaji.unwrap_or_default();

			let en = title.user_preferred.unwrap_or_default();

			// Format the text for the response
			let text = format!(
				"[{}/{}]({})",
				rj,
				en,
				m.site_url.unwrap_or(DEFAULT_STRING.clone())
			);

			// Append the text to the content string
			content.push_str(text.as_str());

			content.push('\n')
		}

		// Construct the description for the response
		let desc = studio_localised
			.desc
			.replace("$id$", studio.id.to_string().as_str())
			.replace(
				"$fav$",
				studio.favourites.unwrap_or_default().to_string().as_str(),
			)
			.replace(
				"$animation$",
				studio.is_animation_studio.to_string().as_str(),
			)
			.replace("$list$", content.as_str());

		// Retrieve the name of the studio
		let name = studio.name;

		let embed_content = EmbedContent::new(name)
			.description(desc)
			.url(studio.site_url.unwrap_or_default());

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
}
