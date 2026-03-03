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
use anyhow::anyhow;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::studio::{
	StudioQuerryId, StudioQuerryIdVariables, StudioQuerrySearch, StudioQuerrySearchVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "studio", desc = "Info of a studio.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "studio", desc = "Name of the studio you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn studio_command(self_: StudioCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("studio"))
		.ok_or(anyhow!("No studio specified"))?;

	// Fetch the studio's data from AniList
	let studio = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>()?;

		let var = StudioQuerryIdVariables { id: Some(id) };

		let operation = StudioQuerryId::build(var);

		let data: GraphQlResponse<StudioQuerryId> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().studio.unwrap()
	} else {
		let var = StudioQuerrySearchVariables {
			search: Some(value.as_str()),
		};

		let operation = StudioQuerrySearch::build(var);

		let data: GraphQlResponse<StudioQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().studio.unwrap()
	};

	// Get the language identifier for localization
	let lang_id = cx.lang_id().await;

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

	// Construct the description for the response using Fluent
	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("id"),
		FluentValue::from(studio.id.to_string()),
	);
	args.insert(
		Cow::Borrowed("fav"),
		FluentValue::from(studio.favourites.unwrap_or_default().to_string()),
	);
	args.insert(
		Cow::Borrowed("animation"),
		FluentValue::from(studio.is_animation_studio.to_string()),
	);
	args.insert(Cow::Borrowed("list"), FluentValue::from(content.as_str()));

	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "anilist_user_studio-desc", &args);

	// Retrieve the name of the studio
	let name = studio.name;

	let embed_content = EmbedContent::new(name)
		.description(desc)
		.url(studio.site_url.unwrap_or_default());

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
