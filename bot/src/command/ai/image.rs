//! The `ImageCommand` structure and implementation encapsulate the logic for generating AI-based images
//! via a bot command interaction with Discord's serenity library. This includes calling a configurable AI service
//! and handling the resulting image data. It also supports saving the generated images to a server or local path.
//!
//! # Fields
//! - `ctx`: The context of the Serenity bot needed to derive bot information or make external HTTP requests.
//! - `command_interaction`: Represents the command interaction object triggering this image command logic.
//! - `command_name`: The specific name of the command being invoked.
//!
//! # Trait Implementations
//! Implements the `Command` trait to handle and parse user-provided options, perform API interactions, and upload responses.
//!
//! ## `get_ctx`
//! Provides a reference to the `SerenityContext`. It's used mostly to retrieve bot state or data.
//!
//! ## `get_command_interaction`
//! Returns a reference to the interaction object to parse user inputs or access server-specific data.
//!
//! ## `get_contents`
//! Main logic for the command. Processes user inputs, interacts with the AI image generation API, and
//! uploads the generated image(s) as a follow-up response.
//!
//! # Helper Functions
//!
//! ## `get_value`
//! Generates the payload required for the AI image generation API, dynamically handling configurations and user inputs.
//!
//! - **Inputs**:
//!   - `command_interaction`: Interaction data for retrieving user-defined command options.
//!   - `n`: Number of images to generate.
//!   - `config`: A reference to the configuration struct to retrieve AI-specific settings.
//!
//! - **Output**: A JSON `Value` object representing the payload for the API.
//!
//! ## `image_with_n_equal_1`
//! Creates an attachment for single image responses.
//!
//! - **Inputs**:
//!   - `filename`: The name for the attachment file.
//!   - `bytes`: Byte data of the single generated image.
//!   
//! - **Output**: A `CreateAttachment` for embedding in the Discord message response.
//!
//! ## `image_with_n_greater_than_1`
//! Creates multiple attachments for when more than one image is requested.
//!
//! - **Inputs**:
//!   - `filename`: A base filename for the generated images (appends index numbers for unique filenames).
//!   - `bytes`: A vector of byte data for generated images.
//!   
//! - **Outputs**: A tuple containing a vector of `CreateAttachment` objects and corresponding filenames.
//!
//! # API Interaction
//! Uses the `reqwest` library to make HTTP POST requests to the AI service, specifying the desired image parameters
//! like prompt, size, and style. Ensures proper authentication and header configuration.
//!
//! # Error Handling
//! Returns Result types for most methods to ensure graceful handling of issues like API errors, byte processing failures,
//! or limits breached on user commands.
use bytes::Bytes;
use std::sync::Arc;

use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandFiles, CommandType, EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::config::Config;
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::{
	get_option_map_integer_subcommand, get_option_map_string_subcommand,
};
use crate::helper::image_saver::general_image_saver::image_saver;
use crate::structure::message::ai::image::load_localization_image;
use anyhow::{Result, anyhow};
use image::EncodableLayout;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serenity::all::{
	AttachmentData, CommandInteraction, Context as SerenityContext, CreateAttachment, EmbedImage,
};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{error, trace};
use uuid::Uuid;

/// A structure representing an image command in the context of a Discord bot.
///
/// This structure is used to encapsulate the necessary data for handling
/// an image-related command issued by a user via interactions in Discord.
///
/// # Fields
/// * `ctx` - The context of the bot, which provides access to the Discord API and
///           other utilities necessary for handling the interaction.
/// * `command_interaction` - Represents the interaction data for the command, including
///                           details about the user who triggered it and how it was invoked.
/// * `command_name` - The name of the image command being executed, as specified by the user.
pub struct ImageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
	pub command_name: String,
}

impl Command for ImageCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) stored within the instance.
	///
	/// # Examples
	/// ```rust
	/// let my_instance = MyStruct { ctx: serenity_context };
	/// let context = my_instance.get_ctx();
	/// ```
	///
	/// This function is useful when access to the `SerenityContext` is required for operations
	/// involving the Serenity library, such as managing Discord bot events or sending messages.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object (`&CommandInteraction`) stored within the struct.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use `interaction` as needed
	/// ```
	///
	/// This method is useful for accessing the stored command interaction details
	/// without taking ownership of the data.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves content by generating images using an AI-based service.
	///
	/// This function performs several tasks in sequence:
	/// 1. Checks if the command's hourly usage limit has been reached for a user.
	/// 2. Constructs the appropriate request data and headers for the AI-based image generation API.
	/// 3. Sends a request to the AI API to generate images based on the user inputs.
	/// 4. Parses the API response, extracts and processes image bytes, and saves the generated images.
	/// 5. Organizes the images into an embed format and prepares the result for further usage.
	///
	/// # Returns
	///
	/// Returns a `Result` containing:
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` — A vector of embed contents containing the generated images.
	/// - `Err(anyhow::Error)` — An error if the process fails at any step.
	///
	/// # Errors
	///
	/// This function returns an error in the following cases:
	/// - The user has reached the hourly usage limit for the command.
	/// - There are issues with the AI API request or response.
	/// - Image data cannot be parsed or saved successfully.
	/// - Configuration or header creation fails unexpectedly.
	///
	/// # Usage
	///
	/// Call this function from within an async context to handle image generation and embed construction:
	///
	/// ```no_run
	/// let contents = my_obj.get_contents().await;
	/// match contents {
	///     Ok(embed_content) => {
	///         // Handle embed content with generated images
	///     }
	///     Err(e) => {
	///         // Handle any errors during the process
	///     }
	/// }
	/// ```
	///
	/// # Steps and Workflow
	///
	/// 1. **Context Setup**:
	///    - Retrieves the bot's shared data (`BotData`) and the command interaction context.
	/// 2. **Hourly Limit Check**:
	///    - Validates if the user has exceeded the hourly limit for the command.
	/// 3. **Prepare API Request**:
	///    - Constructs the request URL, headers, and body using bot configuration and user inputs.
	/// 4. **Send API Request**:
	///    - Sends the image generation request to the AI service and receives a response.
	/// 5. **Process Response**:
	///    - Extracts and processes image bytes from the API response.
	///    - Saves images locally or remotely, based on the bot's settings.
	/// 6. **Construct Embed**:
	///    - Organizes the processed images into a user-friendly embed format.
	///
	/// # Notes
	///
	/// - The service uses unique identifiers (`UUID`) for image filenames to avoid collisions.
	/// - Supports handling single as well as multiple images based on user input.
	/// - Implements localization for image titles based on a guild's settings.
	///
	/// # Dependencies
	///
	/// This function depends on the following:
	/// - `BotData` for shared bot configurations and HTTP client.
	/// - External services (e.g., AI API and image storage services) for image generation and saving.
	/// - Utility functions: `get_option_map_integer_subcommand`, `get_value`, `get_image_from_response`,
	///   `image_with_n_equal_1`, `image_with_n_greater_than_1`, and `image_saver` for intermediary operations.
	///
	/// # Parameters
	///
	/// - `&self`: The instance of the struct invoking this method, which contains necessary contextual data.
	///
	/// # Example Scenario
	///
	/// A user requests image generation by invoking a command. The bot:
	/// 1. Validates the user's remaining request quota.
	/// 2. Prepares and sends a generation request to the AI service.
	/// 3. Processes the response, saves images, and organizes them into an embed.
	/// 4. Returns the embed for further interaction or display as a follow-up.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		if self
			.check_hourly_limit(
				self.command_name.clone(),
				&bot_data.clone(),
				PremiumCommandType::AIImage,
			)
			.await?
		{
			return Err(anyhow!(
				"You have reached your hourly limit. Please try again later.",
			));
		}

		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let map = get_option_map_integer_subcommand(command_interaction);
		let client = bot_data.http_client.clone();

		let n = *map.get(&String::from("n")).unwrap_or(&1);
		let data = get_value(command_interaction, n, &config);
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let image_localised = load_localization_image(guild_id.clone(), config.db.clone());
		self.defer().await?;

		let uuid_name = Uuid::new_v4();
		let filename = format!("{}.png", uuid_name);
		let token = config.ai.image.ai_image_token.clone().unwrap_or_default();
		let token = token.as_str();
		let url = config
			.ai
			.image
			.ai_image_base_url
			.clone()
			.unwrap_or_default();

		let url = if url.ends_with("v1/") {
			format!("{}images/generations", url)
		} else if url.ends_with("v1") {
			format!("{}/images/generations", url)
		} else {
			format!("{}/v1/images/generations", url)
		};

		let mut headers = HeaderMap::new();
		headers.insert(
			AUTHORIZATION,
			HeaderValue::from_str(&format!("Bearer {}", token))?,
		);
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

		let url = url.as_str();
		let res = client.post(url).headers(headers).json(&data).send().await?;
		let res = res.json().await?;

		let guild_id = match command_interaction.guild_id {
			Some(guild_id) => guild_id.to_string(),
			None => String::from("0"),
		};

		let bytes = get_image_from_response(
			res,
			config.image.save_image.clone(),
			config.image.save_server.clone(),
			config.image.token.clone(),
			guild_id.clone(),
		)
		.await?;

		let image_localised = image_localised.await?;
		let embed_content = EmbedContent::new(image_localised.title).description(String::new());

		let mut embed_contents = vec![];
		let mut command_files = vec![];

		if n == 1 {
			let attachment = image_with_n_equal_1(filename.clone(), bytes.clone()).await;
			let name = filename;

			command_files.push(CommandFiles::new(name.clone(), attachment));
			embed_contents.push(
				embed_content
					.clone()
					.images_url(format!("attachment://{}", name.clone())),
			);
		} else {
			let attachements = image_with_n_greater_than_1(filename, bytes).await;
			for attachement in attachements {
				let name = attachement.1;
				let bytes = attachement.0;
				command_files.push(CommandFiles::new(name.clone(), bytes));
				embed_contents.push(
					embed_content
						.clone()
						.images_url(format!("attachment://{}", name.clone())),
				);

				let image_config = bot_data.config.image.clone();
				image_saver(
					guild_id.to_string(),
					name.to_string(),
					bytes,
					image_config.save_server.unwrap_or_default(),
					image_config.token.unwrap_or_default(),
					image_config.save_image,
				)
				.await?;
			}
		};

		let embed_contents = EmbedsContents::new(CommandType::Followup, embed_contents)
			.add_files(command_files)
			.clone();

		Ok(embed_contents)
	}
}

/// Generates a JSON `Value` containing the information required to create an AI-generated image based on the provided input parameters.
///
/// # Arguments
///
/// * `command_interaction` - A reference to a `CommandInteraction` instance that contains the user's command input data.
/// * `n` - An integer specifying the number of image results to generate.
/// * `config` - A reference to an `Arc<Config>` containing configuration settings for the AI image generation.
///
/// # Returns
///
/// Returns a `serde_json::Value` containing the image generation configuration in JSON format.
///
/// The JSON object has the following fields:
/// - `prompt`: The description for the image, derived from the user input or a default value.
/// - `n`: The number of images to generate.
/// - `size`: The output image dimensions, retrieved from configuration or defaults to "1024x1024".
/// - `model`: The AI model to use for image generation, retrieved from configuration or defaults to an empty string.
/// - `quality` (optional): The quality setting for the generated image, if specified in the configuration.
/// - `style` (optional): The style of the generated image, if specified in the configuration.
/// - `response_format`: The response format, statically set to "url".
///
/// # Behavior
///
/// Depending on the configuration provided, the function dynamically adjusts the JSON output:
/// - If both `quality` and `style` are set in the configuration, they will be included in the output JSON.
/// - If only one of `quality` or `style` is set, only the present field will be included.
/// - If neither `quality` nor `style` are set, the JSON excludes these fields.
///
/// # Example Usage
///
/// ```
/// let command_interaction = ...; // CommandInteraction instance from user input.
/// let config = Arc::new(Config::default());
/// let n = 2;
/// let data = get_value(&command_interaction, n, &config);
///
/// println!("{}", data);
/// ```
///
/// The resulting JSON might look like:
/// ```json
/// {
///     "prompt": "A beautiful landscape",
///     "n": 2,
///     "size": "1024x1024",
///     "model": "default_model",
///     "response_format": "url"
/// }
/// ```
/// ```
fn get_value(command_interaction: &CommandInteraction, n: i64, config: &Arc<Config>) -> Value {
	let map = get_option_map_string_subcommand(command_interaction);

	let prompt = map
		.get(&String::from("description"))
		.unwrap_or(DEFAULT_STRING);

	let model = config.ai.image.ai_image_model.clone().unwrap_or_default();

	let model = model.as_str();

	let quality = config.ai.image.ai_image_style.clone();

	let style = config.ai.image.ai_image_quality.clone();

	let size = config
		.ai
		.image
		.ai_image_size
		.clone()
		.unwrap_or(String::from("1024x1024"));

	let data: Value = match (quality, style) {
		(Some(quality), Some(style)) => {
			json!({
				"prompt": prompt,
				"n": n,
				"size": size,
				"model": model,
				"quality": quality,
				"style": style,
				"response_format": "url"
			})
		},
		(None, Some(style)) => {
			json!({
				"prompt": prompt,
				"n": n,
				"size": size,
				"model": model,
				"style": style,
				"response_format": "url"
			})
		},
		(Some(quality), None) => {
			json!({
				"prompt": prompt,
				"n": n,
				"size": size,
				"model": model,
				"quality": quality,
				"response_format": "url"
			})
		},
		(None, None) => {
			json!({
				"prompt": prompt,
				"n": n,
				"size": size,
				"model": model,
				"response_format": "url"
			})
		},
	};

	data
}

/// Asynchronously creates an attachment from a single image with `n` equal to 1.
///
/// This function assumes that the input `bytes` vector contains exactly one `Bytes` object,
/// and it extracts the raw byte data from the first element to create an attachment using
/// the provided `filename`.
///
/// # Generic Parameters
///
/// - `'a`: The lifetime parameter for the created attachment.
///
/// # Parameters
///
/// - `filename`: A string representing the file name for the attachment.
/// - `bytes`: A vector of `Bytes` where the first element's byte data will be used to
///   create the attachment. It is expected that the vector contains at least one element,
///   as this function accesses `bytes[0]`.
///
/// # Returns
///
/// Returns a `CreateAttachment<'a>` containing the byte data from the first element of
/// the `bytes` vector, associated with the given `filename`.
///
/// # Panics
///
/// This function will panic if the `bytes` vector is empty, as it tries to access `bytes[0]`.
///
/// # Example
///
/// ```rust
/// use some_crate::CreateAttachment;
/// use bytes::Bytes;
///
/// async fn example() {
///     let filename = String::from("image.png");
///     let data = Bytes::from_static(b"image data");
///
///     let attachment = image_with_n_equal_1(filename, vec![data]).await;
///     // Use the `attachment` as needed.
/// }
/// ```
async fn image_with_n_equal_1(filename: String, bytes: Vec<Bytes>) -> Vec<u8> {
	let bytes = bytes[0].as_bytes().to_vec();

	bytes
}

/// Processes a vector of images and generates attachments with unique filenames.
///
/// This asynchronous function takes a filename and a vector of `Bytes`,
/// and associates each byte slice with a unique filename by appending an index.
/// It then creates a list of `CreateAttachment` objects and corresponding filenames.
///
/// # Arguments
///
/// * `filename` - A `String` which serves as the base name for the generated files.
/// * `bytes` - A vector of `Bytes` where each element represents an image's data.
///
/// # Returns
///
/// A tuple containing:
/// 1. A vector of `CreateAttachment` objects, each containing the respective byte data and unique filename.
/// 2. A vector of `String` where each entry is the unique filename corresponding to the `CreateAttachment`.
///
/// # Example
///
/// ```rust
/// use bytes::Bytes;
///
/// let filename = "image".to_string();
/// let image_data = vec![Bytes::from(vec![1, 2, 3]), Bytes::from(vec![4, 5, 6])];
///
/// let (attachments, filenames) = image_with_n_greater_than_1(filename, image_data).await;
///
/// assert_eq!(filenames, vec!["image_0_0.png", "image_0_1.png"]);
/// // `attachments` will contain the corresponding `CreateAttachment` objects
/// ```
///
/// # Note
///
/// Ensure that the function is called with a vector of `Bytes` that contains at least one element,
/// as the function assumes the presence of valid byte data.
///
/// # Errors
///
/// No specific errors are handled within this function. It is assumed that the input
/// byte data is correctly formatted and suitable for attachment creation.
///
async fn image_with_n_greater_than_1<'a>(
	filename: String, bytes: Vec<Bytes>,
) -> Vec<(Vec<u8>, String)> {
	let attachments: Vec<(Vec<u8>, String)> = bytes
		.iter()
		.enumerate()
		.map(|(index, byte)| {
			let filename = format!("{}_{}.png", filename, index);
			let byte = byte.as_bytes().to_vec();
			(byte, format!("{}_{}.png", filename, index))
		})
		.collect();

	attachments
}

/// Asynchronously retrieves images from a JSON response, processes them, and optionally saves them to a server.
///
/// # Parameters
/// - `json` (`Value`): A JSON object that contains the data from which image URLs will be extracted.
/// - `saver_server` (`String`): The server URL to which the images will be saved, if specified.
/// - `token` (`Option<String>`): An optional token for authentication on the saver server.
/// - `save_type` (`Option<String>`): An optional parameter specifying the save method/type.
/// - `guild_id` (`String`): The identifier of the guild associated with the operation.
///
/// # Returns
/// Returns a `Result` containing:
/// - `Ok(Vec<Bytes>)`: A vector of binary data (`Bytes`) for each downloaded image if successful.
/// - `Err`: An error if something goes wrong during the process (e.g., JSON parsing, image download, or saving error).
///
/// # Behavior
/// 1. Parses the input JSON to extract image-related data.
/// 2. Retrieves image URLs from the parsed JSON data.
/// 3. Fetches each image URL concurrently.
/// 4. Saves the images to a remote server (if applicable) using the provided `saver_server` and `token`.
/// 5. Collects and returns the binary image data as a vector of `Bytes`.
///
/// # Errors
/// The function may return an error in these cases:
/// - If deserialization of the JSON fails.
/// - If the images cannot be downloaded from their URLs.
/// - If saving the images to the remote server fails.
///
/// # Example
/// ```rust
/// use serde_json::json;
/// use bytes::Bytes;
///
/// #[tokio::main]
/// async fn main() {
///     let json = json!({
///         "data": [
///             { "url": "https://example.com/image1.png" },
///             { "url": "https://example.com/image2.png" }
///         ]
///     });
///     
///     let saver_server = "https://saver-server.example.com".to_string();
///     let token = Some("example-token".to_string());
///     let save_type = Some("type1".to_string());
///     let guild_id = "12345".to_string();
///     
///     match get_image_from_response(json, saver_server, token, save_type, guild_id).await {
///         Ok(images) => println!("Successfully retrieved {} images.", images.len()),
///         Err(e) => eprintln!("Failed to retrieve images: {}", e),
///     }
/// }
/// ```
///
/// # Notes
/// - This function leverages the `reqwest` library to perform HTTP requests.
/// - Image saving is handled by the `image_saver` function, which must be implemented separately.
/// - Logging calls such as `trace!` and `error!` are used for debug and error reporting as part of system tracing.
///
/// # Dependencies
/// - `serde_json`: For JSON parsing.
/// - `reqwest`: For HTTP client functionality.
/// - `bytes`: For handling binary data.
/// - `uuid`: For generating unique file names for the images.
/// - `anyhow`: For error handling.
///
/// # Logging
/// - Logs the extracted URLs at the `trace` level.
/// - Logs errors related to saving images at the `error` level.
async fn get_image_from_response(
	json: Value, saver_server: String, token: Option<String>, save_type: Option<String>,
	guild_id: String,
) -> Result<Vec<Bytes>> {
	let token = token.unwrap_or_default();

	let saver = save_type.unwrap_or_default();

	let mut bytes = Vec::new();

	let root: Root = match serde_json::from_value(json.clone()) {
		Ok(root) => root,
		Err(e) => {
			let root1: Root1 = serde_json::from_value(json)?;

			return Err(anyhow!(format!(
				"Error: {} ............ {:?}",
				e, root1.error
			)));
		},
	};

	let urls: Vec<String> = root.data.iter().map(|data| data.url.clone()).collect();

	trace!("{:?}", urls);

	for (i, url) in urls.iter().enumerate() {
		let client = reqwest::Client::new();

		let res = client.get(url).send().await?;

		let body = res.bytes().await?;

		let filename = format!("ai_{}_{}.png", i, Uuid::new_v4());

		match image_saver(
			guild_id.clone(),
			filename.clone(),
			Vec::from(body.clone()),
			saver_server.clone(),
			token.clone(),
			saver.clone(),
		)
		.await
		{
			Ok(_) => (),
			Err(e) => error!("Error saving image: {}", e),
		}

		bytes.push(body);
	}

	Ok(bytes)
}

/// The `Root` struct is used to represent the root structure of a deserialized JSON object.
///
/// This struct leverages the `serde` library for serializing and deserializing data and derives
/// the `Debug` and `Deserialize` traits for debugging and JSON deserialization purposes.
///
/// # Fields
///
/// * `data` - A vector containing `Data` elements, representing the data section of the JSON object.
///   This field is annotated with `#[serde(rename = "data")]`, which means it maps to a JSON key named "data".
///
/// # Example JSON Representation
///
/// ```json
/// {
///   "data": [
///     {
///       // fields that correspond to the `Data` struct
///     },
///     {
///       // additional `Data` objects
///     }
///   ]
/// }
/// ```
///
/// In this example, the `data` key in the JSON object corresponds to the `data` field in the `Root` struct.
///
/// # Dependencies
///
/// The struct requires the `serde` crate along with the `serde_derive` procedural macros for the `Deserialize`
/// attribute. Use the following in your Cargo.toml:
///
/// ```toml
/// [dependencies]
/// serde = { version = "1.0", features = ["derive"] }
/// serde_derive = "1.0"
/// ```
#[derive(Debug, Deserialize)]

struct Root {
	#[serde(rename = "data")]
	data: Vec<Data>,
}

/// A struct that represents a data model used for deserialization.
///
/// The `Data` struct is designed to store a single field `url`
/// which is a `String` value. This struct implements the `Debug`
/// and `Deserialize` traits.
///
/// # Attributes
/// * `url` - A `String` that holds a URL value.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use your_crate::Data;
///
/// let json_data = r#"{"url": "https://example.com"}"#;
/// let deserialized_data: Data = serde_json::from_str(json_data).unwrap();
///
/// assert_eq!(deserialized_data.url, "https://example.com");
/// ```
#[derive(Debug, Deserialize)]

struct Data {
	url: String,
}

/// Represents an AI-related error with detailed information.
///
/// This struct is used to define the structure of an error message
/// for an AI-related operation, containing specific details about
/// the error type, message, optional parameter, and code.
///
/// # Fields
///
/// * `message` - A human-readable description of the error.
/// * `error_type` - A string indicating the type of the error. This field is serialized and deserialized
///   with the name `"type"`.
/// * `param` - An optional string that represents the parameter related to the error, if applicable.
/// * `code` - A string representing a specific error code associated with the issue.
///
/// # Traits
///
/// The `AiError` struct derives the following traits:
/// * `Debug` - Allows the struct to be formatted using a debugging representation.
/// * `Serialize` - Enables serialization of the struct into formats such as JSON.
/// * `Deserialize` - Enables deserialization of the struct from formats such as JSON.
///
/// # Usage Example
///
/// ```
/// use serde_json;
/// use your_crate_name::AiError;
///
/// let error_json = r#"
/// {
///     "message": "Invalid API key",
///     "type": "authentication_error",
///     "param": null,
///     "code": "401"
/// }
/// "#;
///
/// let ai_error: AiError = serde_json::from_str(error_json).unwrap();
/// println!("{:?}", ai_error);
/// // Output will display structured error information, e.g.,
/// // AiError { message: "Invalid API key", error_type: "authentication_error", param: None, code: "401" }
/// ```
#[derive(Debug, Serialize, Deserialize)]

struct AiError {
	pub message: String,
	#[serde(rename = "type")]
	pub error_type: String,
	pub param: Option<String>,
	pub code: String,
}

/// A structure `Root1` that serves as a wrapper or container for an instance of an `AiError`.
///
/// # Derives
/// - `Debug`: Enables debugging output for instances of this structure, allowing detailed print statements.
/// - `Serialize`: Provides serialization support, enabling the structure to be converted into formats such as JSON or YAML.
/// - `Deserialize`: Allows deserialization from formats such as JSON or YAML into the structure.
///
/// # Fields
/// - `error` (`AiError`): A public field of type `AiError` that holds error-related information.
///
/// # Example
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Root1 {
///     pub error: AiError,
/// }
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct AiError {
///     pub code: u32,
///     pub message: String,
/// }
///
/// let example_error = AiError { code: 404, message: String::from("Not Found") };
/// let root = Root1 { error: example_error };
/// println!("{:?}", root);
/// ```
#[derive(Debug, Serialize, Deserialize)]

struct Root1 {
	pub error: AiError,
}
