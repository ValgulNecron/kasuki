//! The `AnimeRandomImageCommand` struct is responsible for handling the "random anime image"
//! command interaction in a Discord bot. This command allows users to fetch a random anime-related
//! image from a specified category/type.
//!
//! # Fields
//! - `ctx`: A context object (`SerenityContext`) that provides access to the bot's state, API, and resources.
//! - `command_interaction`: The interaction object (`CommandInteraction`) that encapsulates the information
//!   about the incoming command, such as its arguments and options.
//!
//! This struct implements the `Command` trait to integrate into the bot system.
//!
//! # Methods (via `Command` Trait)
//!
//! - `get_ctx()`
//!   - Returns a reference to the bot's context object (`SerenityContext`).
//!   - Used for accessing global bot state during commands, such as configurations and resources.
//!
//! - `get_command_interaction()`
//!   - Returns a reference to the command interaction object (`CommandInteraction`).
//!   - Provides access to user input, options, and metadata for the command.
//!
//! - `get_contents()`
//!   - Asynchronously resolves the content to be displayed in response to the command.
//!   - Retrieves the image type specified by the user, loads localized configuration, and fetches a random
//!     anime image URL for a specified type.
//!   - Returns a `Vec<EmbedContent>` containing the generated embed details.
//!   - Errors out if the input is invalid, fails to find the `image_type` argument, or network/image fetching fails.
//!
//! ---
//!
//! The `random_image_content` function assists in constructing an image embed response based on the provided input.
//!
//! # Arguments
//! - `image_type` (`&str`): Specifies the type of image to fetch (e.g., "happy", "smile").
//! - `title` (`String`): The title to be displayed in the response embed.
//! - `endpoint` (`&'a str`): The endpoint type (e.g., "sfw", "nsfw") to fetch the image from.
//!
//! # Workflow
//! - Constructs the URL for fetching the random image based on the `image_type` and `endpoint`.
//! - Performs an HTTP GET request to retrieve the image metadata.
//! - Extracts the image URL from the JSON response payload.
//! - Downloads the actual image bytes from the fetched URL.
//! - Generates a unique UUID for the image filename (`.gif`) to avoid collisions.
//! - Creates an attachment (`CreateAttachment`) consisting of the image bytes and assigns the filename.
//! - Builds and returns an `EmbedContent` object containing the image attachment and the specified title.
//!
//! # Errors
//! - Returns an error if the image type is omitted, the API request fails, or the image URL cannot be retrieved.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandFiles, CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime::random_image::load_localization_random_image;
use anyhow::{Result, anyhow};
use image::EncodableLayout;
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateAttachment};
use uuid::Uuid;

/// `AnimeRandomImageCommand` is a structure that represents a command to fetch a random anime image.
///
/// This structure contains the following fields:
///
/// - `ctx: SerenityContext`
///   The context in which the command is executed. This provides access to Discord-related
///   functionality such as sending messages, managing roles, and interacting with guilds.
///
/// - `command_interaction: CommandInteraction`
///   Represents the interaction triggered by the user for this command. It includes all relevant
///   data about the command invocation, such as the channel, user, and command arguments.
///
/// This structure is typically used in the implementation of a bot command
/// that fetches and returns a random anime image when invoked.
///
/// # Example
///
/// ```rust
/// use your_crate::AnimeRandomImageCommand;
///
/// let command = AnimeRandomImageCommand {
///     ctx: some_context,
///     command_interaction: some_interaction,
/// };
///
/// // Use `command` to fetch and handle a random anime image.
/// ```
pub struct AnimeRandomImageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AnimeRandomImageCommand {
	/// Retrieves a reference to the `SerenityContext`.
	///
	/// This method provides access to the `SerenityContext` instance associated with the
	/// current state, allowing interaction with Discord's API and various utilities.
	///
	/// # Returns
	///
	/// A reference to `SerenityContext`.
	///
	/// # Examples
	///
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` to interact with Discord's API.
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance (`&CommandInteraction`) stored within the object.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = my_object.get_command_interaction();
	/// // Use `command_interaction` as needed
	/// ```
	///
	/// This method is typically used to access the underlying `CommandInteraction`
	/// for further processing or querying its data.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves a collection of embed contents based on the command interaction and configuration.
	///
	/// This function performs the following tasks:
	/// 1. Retrieves the bot's shared context and configuration data.
	/// 2. Extracts the image type specified by the user from the interaction command options.
	/// 3. Obtains the guild ID from the command interaction; if unavailable, defaults to "0".
	/// 4. Loads localized random image data based on the guild ID and database configuration.
	/// 5. Defers the initial response to the interaction to allow for processing.
	/// 6. Generates an embed content object with a random image of the specified type and localization data.
	///
	/// ### Returns:
	/// - `Ok(Vec<EmbedContent<'_, '_>>)`:
	///   A vector containing a single `EmbedContent` object, which includes the generated random image and metadata.
	/// - `Err(anyhow::Error)`:
	///   An error if any step in the process fails.
	///
	/// ### Errors:
	/// - Fails if no image type is specified in the command interaction.
	/// - Fails if localized random image data cannot be loaded from the database.
	/// - Fails if the interaction response cannot be deferred.
	///
	/// ### Example:
	/// ```rust
	/// let embed_contents = self.get_contents().await?;
	/// // Process or send the `embed_contents` as needed.
	/// ```
	///
	/// ### Dependencies:
	/// This function relies on other utility functions, such as:
	/// - `get_option_map_string_subcommand` for extracting options.
	/// - `load_localization_random_image` for fetching localized data.
	/// - `random_image_content` for generating embed content.
	///
	/// ### Notes:
	/// - Ensure the context includes proper configuration and database access.
	/// - The `image_type` must be provided in the command interaction options.
	/// - Interaction should support deferrals for asynchronous processing.
	///
	/// ### Parameters:
	/// This function does not take direct parameters but relies on the
	/// `self` context, which includes access to the interaction and bot state.
	///
	/// ### Requirements:
	/// - Async runtime must be active for the method to execute.
	/// - Proper error handling must be in place to manage all possible failures.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the type of image to fetch from the command interaction
		let map = get_option_map_string_subcommand(&command_interaction);

		let image_type = map
			.get(&String::from("image_type"))
			.ok_or(anyhow!("No image type specified"))?;

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let random_image_localised =
			load_localization_random_image(guild_id, config.db.clone()).await?;

		self.defer().await?;

		let embed_contents =
			random_image_content(image_type, random_image_localised.title, "sfw").await?;

		Ok(embed_contents)
	}
}

/// Asynchronously fetches and constructs image content for embedding.
///
/// This function interacts with the Waifu Pics API to fetch a random image
/// based on the given parameters and formats it as an `EmbedContent` for use
/// in an application such as a bot.
///
/// # Parameters
///
/// * `image_type` - A string slice that specifies the type of image to fetch
///   (e.g., `"neko"`, `"smile"`, etc.).
/// * `title` - A `String` representing the title associated with the content.
/// * `endpoint` - A string slice representing the endpoint to call (e.g.,
///   `"sfw"` or `"nsfw"`).
///
/// # Returns
///
/// Returns a `Result` containing:
/// * `Ok(EmbedContent<'static, 'static>)` - A construct representing an
///   embedded content message with the downloaded image included.
/// * `Err` - Returns an error if the fetching process fails at any point,
///   such as:
///   - Failure to contact the Waifu Pics API
///   - Issues with parsing the API response
///   - Failure during image file processing
///
/// # Errors
///
/// This function will return an error in several cases, including but not
/// limited to:
/// - HTTP requests to the API endpoint failing
/// - API returning invalid or unexpected JSON format
/// - Unable to retrieve or process the image
///
/// # Example
///
/// ```rust
/// use your_crate::random_image_content;
///
/// #[tokio::main]
/// async fn main() {
///     let image_type = "neko";
///     let title = String::from("Here is a random neko image!");
///     let endpoint = "sfw";
///
///     match random_image_content(image_type, title, endpoint).await {
///         Ok(embed_content) => {
///             // Use the resulting content (e.g., send as a response to a bot command)
///             println!("Generated embed content!");
///         }
///         Err(e) => {
///             eprintln!("Error fetching image content: {}", e);
///         }
///     }
/// }
/// ```
///
/// # Details
///
/// * The function uses the `reqwest` crate to make HTTP GET requests to the
///   Waifu Pics API and download images.
/// * The response from the API is expected to be in JSON format with a key
///   `"url"` pointing to the image.
/// * A UUID is generated for the downloaded image file to ensure uniqueness.
/// * The response content is built as an `EmbedContent` structure.
///
/// # Dependencies
///
/// Ensure the following crates are added to your `Cargo.toml`:
/// - `reqwest`
/// - `serde` and `serde_json` for JSON parsing
/// - `uuid` for generating unique filenames
/// ```
pub async fn random_image_content(
	image_type: &str, title: String, endpoint: &str,
) -> Result<EmbedsContents> {
	// Construct the URL to fetch the image from
	let url = format!("https://api.waifu.pics/{}/{}", endpoint, image_type);

	// Fetch the image from the URL
	let resp = reqwest::get(&url).await?;

	// Parse the response as JSON
	let json: serde_json::Value = resp.json().await?;

	// Retrieve the URL of the image from the JSON
	let image_url = json["url"]
		.as_str()
		.ok_or(anyhow!("No image found"))?
		.to_string();

	// Fetch the image from the image URL
	let response = reqwest::get(image_url).await?;

	// Retrieve the bytes of the image from the response
	let bytes = response.bytes().await?;

	// Generate a UUID for the filename of the image
	let uuid_name = Uuid::new_v4();

	let filename = format!("{}.gif", uuid_name);

	// Construct the attachment for the image
	let bytes = bytes.as_bytes().to_vec();
	let file = CommandFiles::new(filename.clone(), bytes);

	let embed_content =
		EmbedContent::new(title).images_url(format!("attachment://{}", filename.clone()));

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content])
		.add_file(file)
		.clone();

	Ok(embed_contents)
}
