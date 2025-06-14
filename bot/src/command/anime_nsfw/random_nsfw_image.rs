//! The `AnimeRandomNsfwImageCommand` struct represents a Discord bot command.
//! It is responsible for handling the "Random NSFW Anime Image" command and providing
//! the functionality to process user input, retrieve a random NSFW anime image, and
//! send the response back to the user in the form of an embedded message.
//!
//! # Fields
//!
//! - `ctx`:
//!     The `SerenityContext` for the bot, providing access to the Discord context,
//!     including shared data, HTTP client, and other utilities.
//! - `command_interaction`:
//!     The `CommandInteraction` object representing the interaction triggered
//!     by the user, containing details such as the user's input and the command options.
//!
//! # Trait Implementations
//!
//! ## Command
//!
//! Implements the `Command` trait to provide functionality for running and managing the
//! lifecycle of this specific command.
//!
//! ### Methods
//!
//! #### `get_ctx`
//! Retrieves a reference to the bot's `SerenityContext`.
//!
//! - **Returns**: A reference to the `SerenityContext` instance.
//!
//! #### `get_command_interaction`
//! Retrieves a reference to the `CommandInteraction` associated with the command.
//!
//! - **Returns**: A reference to the `CommandInteraction` object.
//!
//! #### `get_contents`
//! This asynchronous method fetches the contents for a random NSFW anime image based
//! on the command interaction input. The steps include:
//!
//! 1. Reading the bot's configuration and data from the context.
//! 2. Extracting the "image_type" from the command options provided by the user.
//!     - Returns an error (`anyhow::Error`) if the "image_type" is missing.
//! 3. Retrieving the guild ID to load localized strings for the response.
//! 4. Using the `random_image_content` function to fetch the appropriate image based
//!     on the user input and localization settings.
//! 5. Returning the fetched image as an embedded message content.
//!
//! - **Returns**: An `Ok` result containing a vector of `EmbedContent` to represent
//!     the command's response, or an `Err` if any error occurs during processing.
//!
//! ### Error Handling:
//! - Errors are handled using the `anyhow` crate, enabling detailed error propagation.
//!   For example:
//!     - Missing "image_type" input will result in an early return with an error.
//!     - Errors from asynchronous function calls (e.g., fetching from the database) are
//!       propagated upwards.
//!
//! # Usage Example
//!
//! The command is defined for use in a bot's command handling system using Serenity,
//! typically triggered by a Discord slash command. When executed by a user, the command
//! will fetch and return a random NSFW anime image based on the provided "image_type".
//!
//! ```no_run
//! // Example usage of AnimeRandomNsfwImageCommand in the bot's command handler
//! let command = AnimeRandomNsfwImageCommand {
//!     ctx: bot_context.clone(),
//!     command_interaction: user_interaction.clone(),
//! };
//! command.run().await?;
//! ```
//!
//! # Dependencies
//!
//! - `serenity::all`: Provides core components for creating and handling Discord
//!     interactions and messages.
//! - `anyhow`: Used for error handling with rich contexts.
//! - Custom modules such as:
//!     - `command::anime::random_image::random_image_content`: Fetches random images.
//!     - `helper::get_option::subcommand::get_option_map_string_subcommand`: Extracts command options.
//!     - `structure::message::anime_nsfw::random_image_nsfw::load_localization_random_image_nsfw`:
//!       Loads localization strings for the NSFW image response.
//! ```
use anyhow::anyhow;

use crate::command::anime::random_image::random_image_content;
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::EmbedsContents;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime_nsfw::random_image_nsfw::load_localization_random_image_nsfw;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing the command to fetch a random NSFW anime image.
///
/// This struct contains the necessary context and interaction details
/// required to execute the command.
///
/// # Fields
///
/// * `ctx` - An instance of `SerenityContext` providing the framework context
/// that includes information about the bot and Discord.
/// * `command_interaction` - An instance of `CommandInteraction` that contains
/// data about the specific command invoked by a user within Discord.
///
/// # Example
/// ```
/// let command = AnimeRandomNsfwImageCommand {
///     ctx: serenity_context_instance,
///     command_interaction: command_interaction_instance,
/// };
/// ```
pub struct AnimeRandomNsfwImageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AnimeRandomNsfwImageCommand {
	/// Retrieves a reference to the current Serenity context.
	///
	/// The Serenity context (`SerenityContext`) provides access to the Discord bot's
	/// runtime environment and allows interaction with the Discord API.
	///
	/// # Returns
	///
	/// A reference to an instance of `SerenityContext` representing the bot's context.
	///
	/// # Example
	///
	/// ```rust
	/// let context = handler.get_ctx();
	/// // Use the context to interact with Discord, e.g., sending messages or modifying state.
	/// ```
	///
	/// This function is usually called when working within an event handler or other
	/// parts of the code where access to the bot's context is required.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```
	/// let command_interaction = instance.get_command_interaction();
	/// // Use `command_interaction` as needed
	/// ```
	///
	/// This function is useful for accessing and utilizing the command interaction data
	/// stored within the instance without taking ownership of it.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves a vector of `EmbedContent` based on the interaction data provided by the caller.
	///
	/// # Returns
	/// A `Result` containing a `Vec<EmbedContent>` on success or an error if the operation fails.
	///
	/// # Process Flow
	/// 1. Fetches the context and bot data needed for processing.
	/// 2. Extracts the command options provided in the interaction, specifically the `image_type`.
	/// 3. Retrieves the `guild_id` from the interaction or defaults to "0" if not found.
	/// 4. Loads the localized random NSFW image strings using the `guild_id` and database configuration.
	/// 5. Sends a deferred response to the interaction to indicate that the bot is working on the request.
	/// 6. Generates an `EmbedContent` object with the randomized NSFW image for the specified `image_type`.
	///
	/// # Errors
	/// - Returns an `anyhow::Error` if:
	///   - The `image_type` option is not provided in the command interaction.
	///   - There is an issue with loading localized random NSFW image strings.
	///   - The deferred response or random image content creation fails.
	///
	/// # Examples
	/// ```rust
	/// let contents = instance.get_contents().await?;
	/// for content in contents {
	///     println!("{:?}", content);
	/// }
	/// ```
	///
	/// # Notes
	/// - This function involves asynchronous calls and interacts with external systems (e.g., database retrieval).
	/// - It is assumed that the bot has appropriate permissions to handle NSFW content safely and within the guidelines of the platform.
	///
	/// # Dependencies
	/// - This method relies on utility functions:
	///   - `get_option_map_string_subcommand`: Extracts command options.
	///   - `load_localization_random_image_nsfw`: Loads localized strings for NSFW random images.
	///   - `random_image_content`: Constructs the `EmbedContent` object.
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the type of image to fetch from the command interaction
		let map = get_option_map_string_subcommand(&command_interaction);

		let image_type = map
			.get(&String::from("image_type"))
			.ok_or(anyhow!("No image type specified"))?;

		let image_type = image_type.clone();

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized random NSFW image strings
		let random_image_nsfw_localised =
			load_localization_random_image_nsfw(guild_id, config.db.clone()).await?;

		// Create a deferred response to the command interaction
		self.defer().await?;

		// Send the random NSFW image as a response to the command interaction
		random_image_content(image_type, random_image_nsfw_localised.title, "nsfw").await
	}
}
