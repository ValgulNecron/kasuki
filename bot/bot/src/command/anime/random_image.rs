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
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandFiles, CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use kasuki_macros::slash_command;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use image::EncodableLayout;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use tracing::{debug, error, info};
use unic_langid::LanguageIdentifier;
use uuid::Uuid;

#[slash_command(
	name = "random_image", desc = "Get a random anime image.",
	command_type = SubCommand(parent = "random_anime"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "image_type", desc = "Type of the image you want.", arg_type = String, required = true, autocomplete = false,
		choices = [(name = "waifu"), (name = "neko"), (name = "shinobu"), (name = "megumin"), (name = "bully"), (name = "cuddle"), (name = "cry"), (name = "hug"), (name = "awoo"), (name = "kiss"), (name = "lick"), (name = "pat"), (name = "smug"), (name = "blush"), (name = "smile"), (name = "wave"), (name = "highfive"), (name = "nom"), (name = "bite"), (name = "slap"), (name = "kill"), (name = "kick"), (name = "happy"), (name = "wink"), (name = "dance")])],
)]
async fn anime_random_image_command(self_: AnimeRandomImageCommand) -> Result<EmbedsContents<'_>> {
		info!("Processing random anime image command");
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let _config = bot_data.config.clone();

		debug!("Retrieving bot data and configuration");

		// Retrieve the type of image to fetch from the command interaction
		debug!("Extracting image type from command options");
		let map = get_option_map_string_subcommand(&command_interaction);

		let image_type = map
			.get(&String::from("image_type"))
			.ok_or_else(|| {
				error!("No image type specified in command options");
				anyhow!("No image type specified")
			})?;

		let image_type = image_type.clone();
		debug!("Requested image type: {}", image_type);

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

		debug!("Loading localization for guild: {}", guild_id);
		let lang = get_guild_language(guild_id.clone(), db_connection).await;
		let lang_code = match lang.as_str() {
			"jp" => "ja",
			"en" => "en-US",
			other => other,
		};
		let lang_id = LanguageIdentifier::from_str(lang_code)
			.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
		let title = USABLE_LOCALES.lookup(&lang_id, "anime_random_image-title");
		debug!("Localization loaded successfully");

		debug!("Deferring command response");
		let _ = self_.defer().await;
		debug!("Command response deferred successfully");

		debug!("Fetching random image content for type: {}", image_type);
		random_image_content(image_type, title, "sfw").await
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
pub async fn random_image_content<'a>(
	image_type: String, title: String, endpoint: &'a str,
) -> Result<EmbedsContents<'a>> {
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

	let mut embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
	embed_contents.add_files(vec![file]);

	Ok(embed_contents)
}
