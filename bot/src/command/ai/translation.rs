//! Handles AI-based translation commands, including video and audio file translations to text.
//!
//! # Structures and Methods
//!
//! ## `TranslationCommand`
//! A struct that implements the `Command` trait and handles the translation logic.
//!
//! ## Fields:
//! - `ctx`: The Serenity `Context` used for bot operations and data access.
//! - `command_interaction`: The interaction context of the command received from the user.
//! - `command_name`: The name of the command.
//!
//! ## Methods in `TranslationCommand`:
//!
//! ### `get_ctx(&self) -> &SerenityContext`
//! Retrieves the reference to the bot's context for performing operations.
//!
//! ### `get_command_interaction(&self) -> &CommandInteraction`
//! Retrieves the reference to the user's command interaction.
//!
//! ### `get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>>`
//! Asynchronously executes the core logic for the translation command:
//! - Validates the user's API rate limit for the command.
//! - Processes user-provided language input, file type, and attachment.
//! - Ensures only supported audio and video formats (`mp3`, `mp4`, `wav`, etc.) are processed.
//! - Sends the user's media file for transcription to the AI transcription API.
//! - Optionally translates the transcription to a target language using another AI translation API (if the target language is not English).
//! - Returns rich embed content with the translation result for response.
//!
//! # Function
//!
//! ## `translation(lang: String, text: String, api_key: String, api_url: String, model: String) -> Result<String>`
//! Asynchronously translates a given text from English to another language based on the ISO-639-1 `lang` code.
//!
//! ### Parameters:
//! - `lang`: The ISO-639-1 language code (e.g., `en`, `es`).
//! - `text`: The text that needs to be translated.
//! - `api_key`: The API key for the translation service.
//! - `api_url`: The base URL of the AI translation service.
//! - `model`: The AI model to be used for translation.
//!
//! ### Returns:
//! - `Ok(String)`: The translated text string if the operation succeeds.
//! - `Err`: Returns an error if the request or processing fails.
//!
//! ### Details:
//! - Utilizes OpenAI's GPT or similar AI to perform context-aware translation.
//! - Sends a translation prompt containing the source language and text to the AI API.
//! - Processes and formats the response to remove unnecessary escape sequences.
//!
//! # Errors
//! - Returns an error if:
//!   - The user exceeds their hourly command limits.
//!   - The provided file type or file extension is unsupported.
//!   - API calls to transcription or translation services fail.
//!   - Required configuration credentials are missing.
//!
//! # Notes
//! - The command relies on features like `command_traits`, `embed`, and bot-specific configurations.
//! - The transcription API call processes the audio or video file into transcribed text, which can later be passed into the translation API if required.
//! - File extensions supported include `mp3`, `mp4`, `mpeg`, `wav`, `webm`, etc.
//! - If the target language is English, the transcription text is directly displayed without further translation.
//!
//! # Example Usage
//! - User uploads an audio file and specifies the target language `es` (Spanish):
//!   - The command first transcribes speech from the media file using the configured transcription API.
//!   - The resulting text is translated into Spanish using the translation API.
//!
//! - User interacts with the bot in a Discord server to get translations.
//!
//! # Dependencies
//! This logic relies on external libraries and APIs:
//! - `reqwest` for HTTP requests.
//! - `serde_json` for parsing API responses.
//! - `serenity` for Discord context.
//! - `anyhow` for simplified error handling.
//! - `tracing` for debugging logs.
//!
//! # Logging
//! - The `trace` macro is used for logging during different stages of command execution.

use crate::command::ai::question::question_api_url;
use crate::command::command::EmbedContent;
use crate::command::command::{Command, CommandRun, EmbedType, PremiumCommand, PremiumCommandType};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::{
	get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use crate::structure::message::ai::translation::load_localization_translation;
use anyhow::{Result, anyhow};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Url, multipart};
use serde_json::{Value, json};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::sync::Arc;
use tracing::trace;
use uuid::Uuid;

///
/// Represents a translation command within a Discord bot.
///
/// This structure encapsulates the context and interaction details
/// required for handling a command related to translation functionality.
///
/// # Fields
///
/// * `ctx` - The context of the bot, represented by `SerenityContext`,
///   which provides access to the bot's state and utility functions.
///
/// * `command_interaction` - The specific interaction instance,
///   captured as `CommandInteraction`, which contains details
///   about the user's interaction with the bot.
///
/// * `command_name` - A `String` representing the name of the command,
///   which indicates the specific translation action to be taken.
///
pub struct TranslationCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,

	pub command_name: String,
}

impl Command for TranslationCommand {
	/// Returns a reference to the `SerenityContext`.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) stored in the
	/// current instance. This allows access to the context object for performing
	/// various operations related to Discord bot functionality.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` to perform actions within the bot's environment.
	/// ```
	///
	/// # Notes
	/// - This method borrows the internal `SerenityContext` as an immutable reference,
	/// so it cannot be used to modify the context directly.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance stored within the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance (`&CommandInteraction`).
	///
	/// # Example
	/// ```
	/// let command_interaction = object.get_command_interaction();
	/// // Use the `command_interaction` reference as needed
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes file contents for transcription and optional translation.
	///
	/// # Description
	/// This function executes the following sequence of operations:
	/// 1. Validates the hourly usage limit for the provided command type.
	/// 2. Extracts options and attachments from the command interaction.
	/// 3. Validates the content type and file extension of the input attachment.
	/// 4. Processes the file for transcription using an AI transcription service.
	/// 5. Optionally translates the transcribed text to a configured target language.
	/// 6. Builds and returns a response struct containing the processed content.
	///
	/// # Pre-requisites
	/// - The user must not exceed the hourly usage limit for the specified command type.
	/// - A valid audio or video file must be provided.
	/// - Proper API tokens and configuration values are required for AI services like transcription and translation.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` containing content data ready for embedding.
	/// - `Err` if any error occurs during the process, such as invalid file, API call failures, or configuration issues.
	///
	/// # Errors
	/// The function can return errors in the following cases:
	/// - Hourly usage limit for the command type is exceeded.
	/// - Missing or invalid required attachment (video/audio file).
	/// - Invalid file type or unsupported file extension.
	/// - Failure to download or process the attachment.
	/// - API errors during transcription or translation.
	/// - Missing or incorrect API configuration or tokens.
	///
	/// # Examples
	/// Given a valid file attachment and proper API configuration, the function can:
	/// - Transcribe an audio file to text.
	/// - Translate the transcribed text into a target language if needed.
	/// - Build an embed content struct for user feedback.
	///
	/// ```
	/// let result = your_instance.get_contents().await;
	/// match result {
	///     Ok(contents) => {
	///         for content in contents {
	///             println!("Embed Title: {}", content.title);
	///             println!("Embed Description: {}", content.description.unwrap());
	///         }
	///     },
	///     Err(error) => eprintln!("Error: {}", error),
	/// }
	/// ```
	///
	/// # Notes
	/// - The transcription API base URL is validated and formatted, ensuring it ends with the correct endpoint.
	/// - Language translation is only performed if the target language is not English.
	/// - The response from the transcription API is parsed assuming a JSON structure with a `text` field for the transcribed content.
	/// - The function defers a response to the user until processing is complete.
	///
	/// # Dependencies
	/// - `reqwest` for HTTP requests to download files and call external APIs.
	/// - `chrono`, `uuid` for generating unique file names.
	/// - Custom data structures, enums, and configurations like `BotData`, `EmbedContent`, `PremiumCommandType`, etc., are used for logical operations.
	/// - Localization and translation utilities to handle various language-specific use cases.
	///
	/// # Thread Safety
	/// This function is marked as asynchronous (`async`) and is designed to be executed within an asynchronous runtime. Ensure that concurrency considerations are respected when invoking this function across multiple tasks.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();

		if self
			.check_hourly_limit(
				self.command_name.clone(),
				&bot_data,
				PremiumCommandType::AITranslation,
			)
			.await?
		{
			return Err(anyhow!(
				"You have reached your hourly limit. Please try again later.",
			));
		}

		self.defer().await?;

		let map = get_option_map_string_subcommand(command_interaction);
		let attachment_map = get_option_map_attachment_subcommand(command_interaction);

		let lang = map
			.get(&String::from("lang"))
			.unwrap_or(DEFAULT_STRING)
			.clone();
		let attachment = attachment_map
			.get(&String::from("video"))
			.ok_or(anyhow!("No option for video"))?;
		let content_type = attachment
			.content_type
			.clone()
			.ok_or(anyhow!("Failed to get the content type"))?;
		let content = attachment.proxy_url.clone();

		if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
			return Err(anyhow!("Unsupported file type"));
		}
		let allowed_extensions = ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm", "ogg"];
		let parsed_url = Url::parse(content.as_str())?;
		let mut path_segments = parsed_url
			.path_segments()
			.ok_or(anyhow!("Failed to get the path segments"))?;
		let last_segment = path_segments.next_back().unwrap_or_default();
		let file_extension = last_segment
			.rsplit('.')
			.next()
			.ok_or(anyhow!("Failed to get the file extension"))?
			.to_lowercase();
		if !allowed_extensions.contains(&&*file_extension) {
			return Err(anyhow!("Unsupported file extension"));
		}

		let token = config
			.ai
			.transcription
			.ai_transcription_token
			.clone()
			.unwrap_or_default();
		let model = config
			.ai
			.transcription
			.ai_transcription_model
			.clone()
			.unwrap_or_default();
		let api_base_url = config
			.ai
			.transcription
			.ai_transcription_base_url
			.clone()
			.unwrap_or_default();

		let response = reqwest::get(content.as_str()).await?; // save the file into a buffer
		let buffer = response.bytes().await?;
		let uuid_name = Uuid::new_v4().to_string();
		let client = reqwest::Client::new();
		let mut headers = HeaderMap::new();
		headers.insert(
			AUTHORIZATION,
			HeaderValue::from_str(&format!("Bearer {}", token))?,
		);

		let api_base_url = if api_base_url.ends_with("v1/") {
			format!("{}audio/translations/", api_base_url)
		} else if api_base_url.ends_with("v1") {
			format!("{}/audio/translations/", api_base_url)
		} else {
			format!("{}/v1/audio/translations/", api_base_url)
		};

		let part = multipart::Part::bytes(buffer.to_vec())
			.file_name(uuid_name)
			.mime_str(content_type.as_str())
			.unwrap();
		let form = multipart::Form::new()
			.part("file", part)
			.text("model", model)
			.text("language", lang.clone())
			.text("response_format", "json");
		let response_result = client
			.post(api_base_url)
			.headers(headers)
			.multipart(form)
			.send()
			.await;
		let response = response_result?;
		let res_result: Result<Value, reqwest::Error> = response.json().await;
		let res = res_result?;
		let text = res["text"].as_str().unwrap_or("");

		let text = if lang != "en" {
			let api_key = config
				.ai
				.question
				.ai_question_token
				.clone()
				.unwrap_or_default();

			let api_base_url = config
				.ai
				.question
				.ai_question_base_url
				.clone()
				.unwrap_or_default();

			let api_base_url = question_api_url(api_base_url);

			let model = config
				.ai
				.question
				.ai_question_model
				.clone()
				.unwrap_or_default();

			translation(
				lang,
				text.to_string(),
				api_key,
				api_base_url,
				model,
				bot_data.http_client.clone(),
			)
			.await?
		} else {
			String::from(text)
		};

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let translation_localised =
			load_localization_translation(guild_id, config.db.clone()).await?;

		let embed_content = EmbedContent::new(translation_localised.title)
			.description(text.to_string())
			.command_type(EmbedType::Followup);

		Ok(vec![embed_content])
	}
}

/// Translates a given text into the target language specified by its ISO-639-1 code using an external translation API.
///
/// # Arguments
///
/// * `lang` - A `String` representing the ISO-639-1 language code (e.g., "es" for Spanish, "fr" for French).
/// * `text` - A `String` containing the text to be translated.
/// * `api_key` - A `String` which serves as the API key for authenticating the translation service.
/// * `api_url` - A `String` representing the URL endpoint of the translation API.
/// * `model` - A `String` indicating the model to be used for translation in the external API.
///
/// # Returns
///
/// This function returns a `Result<String>`. On success, the function returns the translated text as a `String`.
/// On failure, it returns an error that includes details about what went wrong during the API request or response parsing.
///
/// # Example
///
/// ```rust
/// use your_crate_name::translation;
///
/// #[tokio::main]
/// async fn main() {
///     let lang = "es".to_string();
///     let text = "Hello, how are you?".to_string();
///     let api_key = "your_api_key".to_string();
///     let api_url = "https://api.example.com/translate".to_string();
///     let model = "gpt-3.5-turbo".to_string();
///
///     match translation(lang, text, api_key, api_url, model).await {
///         Ok(translated_text) => println!("Translated text: {}", translated_text),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
///
/// # Errors
///
/// - This function can fail due to network issues, e.g., if the API URL is unreachable.
/// - It may fail if the provided API key is invalid or unauthorized.
/// - If the API response is not in the expected format or lacks certain fields, it will result in a parsing error.
///
/// # Implementation Details
///
/// - The function constructs a request payload in JSON format with the input text and model specifications.
/// - The `prompt` is designed specifically for translation tasks, instructing the API to only perform translations.
/// - The API response is parsed to extract the translated text from the JSON structure, specifically under `choices[0]["message"]["content"]`.
/// - Any escaped newlines in the response are replaced with actual line breaks for better readability.
pub async fn translation(
	lang: String, text: String, api_key: String, api_url: String, model: String,
	http_client: Arc<reqwest::Client>,
) -> Result<String> {
	let prompt_gpt = format!("
            i will give you a text and a ISO-639-1 code and you will translate it in the corresponding language
            iso code: {}
            text:
            {}
            ", lang, text);

	let client = http_client.clone();

	let mut headers = HeaderMap::new();

	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
	);

	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

	let data = json!({
		 "model": model,
		 "messages": [{"role": "system", "content": "You are a expert in translating and only do that."},{"role": "user", "content": prompt_gpt}]
	});

	let res: Value = client
		.post(api_url)
		.headers(headers)
		.json(&data)
		.send()
		.await?
		.json()
		.await?;

	let content = res["choices"][0]["message"]["content"].to_string();

	let no_quote = content.replace('"', "");

	Ok(no_quote.replace("\\n", " \n "))
}
