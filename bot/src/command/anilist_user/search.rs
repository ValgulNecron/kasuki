//! The `SearchCommand` is designed to handle the search functionality for AniList-related data types
//! in a Discord bot. It determines the type of search based on the provided user input
//! (e.g., searching for anime, characters, manga, etc.), delegates the task to the respective command,
//! and aggregates the response.
//!
//! # Fields
//!
//! * `ctx` - The `SerenityContext` object provides the context for interacting with the Discord API.
//! * `command_interaction` - The `CommandInteraction` object contains the interaction details from the user.
//!
//! # Implementations
//!
//! Implements the `Command` trait, which provides the following methods:
//!
//! ## Methods
//!
//! ### `get_ctx`
//!
//! ```rust
//! fn get_ctx(&self) -> &SerenityContext;
//! ```
//!
//! Returns a reference to the `ctx` (Discord API context).
//!
//! ### `get_command_interaction`
//!
//! ```rust
//! fn get_command_interaction(&self) -> &CommandInteraction;
//! ```
//!
//! Returns a reference to the `command_interaction` (interaction details).
//!
//! ### `get_contents`
//!
//! ```rust
//! async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>>;
//! ```
//!
//! Handles the search operation based on the type specified by the user in the `command_interaction`.
//! It performs the following:
//!
//! 1. Extracts the user-specified type of search from the command options (e.g., `anime`, `character`, etc.).
//! 2. Matches the type and initializes the corresponding command struct (e.g., `AnimeCommand`, `MangaCommand`, etc.).
//! 3. Executes the `get_contents` method of the associated command to retrieve the results.
//!
//! # Error Handling
//!
//! * If the `type` parameter is not specified, an error is returned.
//! * If the specified `type` is not supported or invalid, an error is returned.
//! * Any errors occurring during the search or execution of the inner command are propagated as part of the `Result`.
//!
//! # Example
//!
//! Assuming a user issues a command to search for an anime:
//!
//! ```rust
//! let search_cmd = SearchCommand {
//!     ctx: SerenityContext { /* .. */ },
//!     command_interaction: CommandInteraction { /* .. */ },
//! };
//!
//! // Asynchronously fetch search results
//! let result = search_cmd.get_contents().await;
//!
//! match result {
//!     Ok(contents) => {
//!         // Process and display the search results
//!     }
//!     Err(e) => {
//!         // Handle errors (e.g., invalid search type specified)
//!     }
//! }
//! ```
//!
//! This command is useful for dynamically searching AniList data types based on user input in a Discord bot.
use anyhow::{Result, anyhow};

use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

use crate::command::anilist_user::anime::AnimeCommand;
use crate::command::anilist_user::character::CharacterCommand;
use crate::command::anilist_user::ln::LnCommand;
use crate::command::anilist_user::manga::MangaCommand;
use crate::command::anilist_user::staff::StaffCommand;
use crate::command::anilist_user::studio::StudioCommand;
use crate::command::anilist_user::user::UserCommand;
use crate::command::command::{Command, CommandRun, EmbedContent};
use crate::helper::get_option::command::get_option_map_string;

/// A struct representing a search command within a Discord bot context.
///
/// This struct contains the necessary context and interaction data
/// to process and handle a search command issued by a user.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext` object, which provides access to the underlying
///           library's context, including data shared across the bot, as well as
///           utility functions for interacting with Discord's API.
/// * `command_interaction` - The `CommandInteraction` object, which includes details
///                            about the specific command interaction, such as its name,
///                            arguments, and the originating user and channel.
///
/// # Usage
///
/// This struct is typically used to encapsulate the context and details
/// needed to handle a command within a Discord bot built using the `serenity` library.
///
pub struct SearchCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SearchCommand {
	/// Retrieves a reference to the stored Serenity context.
	///
	/// This method allows access to the Serenity framework's context,
	/// which provides various utilities and information about the bot
	/// and its environment.
	///
	/// # Returns
	/// A reference to the `SerenityContext` instance stored within the
	/// struct.
	///
	/// # Example
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use the context for further operations
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction`
	/// associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` stored within the instance.
	///
	/// # Examples
	/// ```rust
	/// let instance = MyStruct {
	///     command_interaction: CommandInteraction::new(),
	/// };
	/// let command = instance.get_command_interaction();
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves a list of contents from AniList based on the type specified in the command interaction.
	///
	/// # Returns
	///
	/// A [`Result`] containing:
	/// - On success: A [`Vec`] of [`EmbedContent`] structs, representing the requested data.
	/// - On error: An error indicating why the content could not be retrieved.
	///
	/// # Process
	/// 1. Fetches the context and command interaction tied to the current execution.
	/// 2. Extracts the type of AniList data to search for from the command's options (e.g., anime, manga, etc.).
	/// 3. Matches the extracted type string to determine which specific command handler to use:
	///    * `AnimeCommand` for anime-related searches
	///    * `CharacterCommand` for character-related searches
	///    * `LnCommand` for light novel-related searches
	///    * `MangaCommand` for manga-related searches
	///    * `StaffCommand` for staff-related searches
	///    * `UserCommand` for user-related searches
	///    * `StudioCommand` for studio-related searches
	/// 4. If the type doesn't match a known category, an error is returned.
	///
	/// # Arguments
	/// - `self`: An instance of the type implementing the method.
	///
	/// # Errors
	/// - Returns an error if:
	///   * No type is specified in the options.
	///   * An invalid or unsupported type is provided.
	///   * The corresponding command handler fails to retrieve the requested contents.
	///
	/// # Asynchronous Behavior
	/// This function is asynchronous and should be awaited when called.
	///
	/// # Example Usage
	/// ```
	/// let contents = instance.get_contents().await;
	/// match contents {
	///     Ok(embed_contents) => { /* Process retrieved contents */ }
	///     Err(e) => { /* Handle error */ }
	/// }
	/// ```
	///
	/// # Dependencies
	/// - Relies on the following dynamic command handlers:
	///   - `AnimeCommand`
	///   - `CharacterCommand`
	///   - `LnCommand`
	///   - `MangaCommand`
	///   - `StaffCommand`
	///   - `UserCommand`
	///   - `StudioCommand`
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();

		// Retrieve the type of AniList data to search for from the command interaction
		let map = get_option_map_string(command_interaction);

		let search_type = map
			.get(&FixedString::from_str_trunc("type"))
			.ok_or(anyhow!("No type specified"))?;

		// Execute the corresponding search function based on the specified type
		let dyn_cmd: dyn Command = match search_type.as_str() {
			"anime" => AnimeCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			"character" => CharacterCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			"ln" => LnCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			"manga" => MangaCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			"staff" => StaffCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			"user" => UserCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			"studio" => StudioCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			},
			// Return an error if the specified type is not one of the expected types
			_ => return (Err(anyhow!("Type does not exist."))),
		};

		dyn_cmd.get_contents().await
	}
}
