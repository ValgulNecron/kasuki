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
use crate::command::command::CommandRun;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use tracing::{debug, error, info};
use unic_langid::LanguageIdentifier;

#[slash_command(
	name = "random_himage", desc = "Get a random nsfw anime image.",
	command_type = SubCommand(parent = "random_hanime"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "image_type", desc = "Type of the image you want.", arg_type = String, required = true, autocomplete = false,
		choices = [(name = "waifu"), (name = "neko"), (name = "trap")])],
)]
async fn anime_random_nsfw_image_command(
	self_: AnimeRandomNsfwImageCommand,
) -> Result<EmbedsContents<'_>> {
	info!("Processing random NSFW anime image command");
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let _config = bot_data.config.clone();

	debug!("Retrieving bot data and configuration");

	// Retrieve the type of image to fetch from the command interaction
	debug!("Extracting image type from command options");
	let map = get_option_map_string_subcommand(&command_interaction);

	let image_type = map.get(&String::from("image_type")).ok_or_else(|| {
		error!("No image type specified in command options");
		anyhow!("No image type specified")
	})?;

	let image_type = image_type.clone();
	debug!("Requested NSFW image type: {}", image_type);

	// Retrieve the guild ID from the command interaction
	let guild_id = match command_interaction.guild_id {
		Some(id) => {
			debug!("Command executed in guild: {}", id);
			id.to_string()
		},
		None => {
			debug!("Command executed in DM");
			String::from("0")
		},
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized random NSFW image strings
	debug!("Loading localization for guild: {}", guild_id);
	let lang = get_guild_language(guild_id.clone(), db_connection).await;
	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
	let title = USABLE_LOCALES.lookup(&lang_id, "anime_nsfw_random_image_nsfw-title");
	debug!("Localization loaded successfully");

	// Create a deferred response to the command interaction
	debug!("Deferring command response");
	debug!("Command response deferred successfully");

	// Send the random NSFW image as a response to the command interaction
	debug!(
		"Fetching random NSFW image content for type: {}",
		image_type
	);
	random_image_content(image_type, title, "nsfw").await
}
