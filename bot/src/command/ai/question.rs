//! A command structure to handle the "Question" functionality in the bot.
//!
//! The `QuestionCommand` struct implements the `Command` trait and is used to process
//! a specific command interaction where the user asks a question to an AI-based service.
//!
//! # Fields
//! - `ctx` - The Serenity context for the bot containing necessary data and utilities.
//! - `command_interaction` - The interaction data from the user, including input options.
//! - `command_name` - The name of the command invoked by the user.
//!
//! # See Also
//! - [`Command`](crate::command::command::Command): The trait this struct implements.

use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use anyhow::{Result, anyhow};
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::sync::Arc;
use tracing::trace;

/// The `QuestionCommand` struct represents a command in the context of a bot using the Serenity framework.
/// It encapsulates the context, interaction details, and the command name specifically related to a user-issued command.
///
/// Fields:
/// - `ctx` (`SerenityContext`): The context in which the command is being executed,
///   providing access to various utilities and resources necessary for handling the command.
///
/// - `command_interaction` (`CommandInteraction`): Represents the interaction object triggered by the user.
///   This contains data related to the specific interaction, such as options provided by the user
///   and the channel or guild where the interaction occurred.
///
/// - `command_name` (`String`): The name of the command as issued by the user,
///   allowing the logic to dynamically reference or process specific command implementations.
pub struct QuestionCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
	pub command_name: String,
}

impl Command for QuestionCommand {
	/// Retrieves a reference to the `SerenityContext` associated with this instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` stored within the current object.
	///
	/// # Usage
	/// This method allows access to the underlying `SerenityContext`, which can be used for
	/// performing operations or retrieving data related to the bot's state or Discord interactions.
	///
	/// # Example
	/// ```rust
	/// let context = my_instance.get_ctx();
	/// // Use `context` to interact with the bot's state or execute actions.
	/// ```
	///
	/// # Notes
	/// Ensure that the lifetime of the returned reference aligns with the
	/// expected usage to avoid borrowing issues.
	///
	/// # Context
	/// The `SerenityContext` is an abstraction provided by the Serenity library
	/// for managing bot operations like cache access, data sharing, and event handling.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` field of the object.
	///
	/// # Example
	/// ```
	/// let interaction = obj.get_command_interaction();
	/// // Use the interaction as needed
	/// ```
	///
	/// # Notes
	/// - This method returns an immutable reference to ensure the `CommandInteraction` cannot be modified.
	/// - Ensure the `CommandInteraction` field is properly initialized before invoking this method.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves content based on user input and processes it through an AI question system.
	///
	/// # Description
	/// This asynchronous function fetches user input from the command interaction,
	/// performs checks (e.g., hourly usage limits), then forwards a prompt to an AI-based
	/// question API and returns the generated content encapsulated in an `EmbedContent` structure.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` - A vector containing the result of the AI question processing in
	///   an embedded format, ready to be presented to the user.
	/// - `Err(anyhow::Error)` - Indicates an error occurred either due to reaching the hourly limit or
	///   during the question processing steps (e.g., API interaction failure).
	///
	/// # Steps
	/// 1. **Context and Configuration Retrieval**: Obtains the bot's execution context, command
	/// interaction data, and associated configuration settings.
	/// 2. **Hourly Limit Check**: Verifies if the current command has exceeded the allowed hourly usage.
	///    - If exceeded, an error with the appropriate message is returned.
	/// 3. **Option Map Retrieval**: Extracts command options, ensuring the "prompt" parameter is fetched
	///    from the command interaction. Defaults to a constant string if `prompt` is not provided.
	/// 4. **API Interaction**: Sends the `prompt` along with the AI API key, base URL, and model
	///    to the question service to return a generated text response.
	/// 5. **Embed Creation**: Wraps the processed text output from the API into an `EmbedContent`
	///    object to format it for later display as a follow-up message.
	///
	/// # Parameters
	/// - This method operates on the instance of the structure implementing it (`&self`).
	///
	/// # API Configuration
	/// The AI API integration uses the following configuration parameters:
	/// - `ai_question_token`: API key for authentication.
	/// - `ai_question_base_url`: The base URL of the AI question service.
	/// - `ai_question_model`: The configured model for generating responses.
	///
	/// # Error Handling
	/// - A rate-limiting error is returned if the command exceeds its hourly limit.
	/// - API errors or missing parameters during interaction with the AI endpoint
	///   will result in an error being propagated.
	///
	/// # Example Usage
	/// ```rust
	/// let contents = instance.get_contents().await;
	/// match contents {
	///     Ok(embed_contents) => {
	///         for embed in embed_contents {
	///             // Process and display embed content
	///         }
	///     }
	///     Err(err) => println!("Error: {:?}", err),
	/// }
	/// ```
	///
	/// # Dependencies
	/// - `BotData`: Provides configuration and state for the bot.
	/// - `EmbedContent`: Represents the structure of embedded content sent back to users.
	/// - External AI service for question generation.
	///
	/// # Notes
	/// - Ensure the API key, base URL, and model are properly configured before invoking
	///   this method.
	/// - The function automatically defers the command interaction to indicate that processing
	///   is ongoing while awaiting a response from the API.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();

		if self
			.check_hourly_limit(
				self.command_name.clone(),
				&bot_data,
				PremiumCommandType::AIQuestion,
			)
			.await?
		{
			return Err(anyhow!(
				"You have reached your hourly limit. Please try again later.",
			));
		}

		let map = get_option_map_string_subcommand(command_interaction);
		let prompt = map.get(&String::from("prompt")).unwrap_or(DEFAULT_STRING);
		self.defer().await?;

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
		let model = config
			.ai
			.question
			.ai_question_model
			.clone()
			.unwrap_or_default();

		let text = question(
			prompt,
			api_key,
			api_base_url,
			model,
			bot_data.http_client.clone(),
		)
		.await?;

		let embed_content = EmbedContent::new(String::new()).description(text);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}

///
/// Sends a text prompt to a specified OpenAI language model API and retrieves the model's response.
///
/// # Parameters
/// - `text`: A reference to the input string containing the user's query or prompt.
/// - `api_key`: A `String` containing the API key for authentication with the OpenAI API.
/// - `api_base_url`: A `String` specifying the base URL of the OpenAI API endpoint.
/// - `model`: A `String` defining the name of the model to be used for processing the input.
///
/// # Returns
/// - Returns a `Result<String, reqwest::Error>`:
///     - `Ok(String)`: The response content generated by the model, with newline escape sequences (`\n`) replaced by actual newlines.
///     - `Err(reqwest::Error)`: An error encountered during the API request or response handling.
///
/// # Errors
/// - Returns an error if:
///     - The `api_url` cannot be constructed properly.
///     - The HTTP request to the API fails (e.g., due to network issues or invalid API key).
///     - The JSON response cannot be parsed correctly or does not contain the expected structure.
///
/// # Example
/// ```rust
/// use your_module::question;
///
/// #[tokio::main]
/// async fn main() {
///     let response = question(
///         &"What is the capital of France?".to_string(),
///         "your_api_key".to_string(),
///         "https://api.openai.com/v1".to_string(),
///         "gpt-3.5-turbo".to_string(),
///     ).await;
///
///     match response {
///         Ok(answer) => println!("Model response: {}", answer),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
///
/// # Notes
/// - The function uses the `reqwest` library for making HTTP requests.
/// - The `Authorization` header is used to authenticate the API call using a bearer token.
/// - The `model` parameter must match an available model on the OpenAI platform (e.g., `gpt-4`, `gpt-3.5-turbo`).
/// - The function assumes a specific JSON structure in the response from the API. If the API's structure changes, the code may need updates.
///
/// # Dependencies
/// - Ensure the following crates are added to your `Cargo.toml`:
/// ```toml
/// reqwest = { version = "0.11", features = ["json", "blocking"] }
/// serde_json = "1.0"
/// log = "0.4"
/// ```
async fn question(
	text: &String, api_key: String, api_base_url: String, model: String, http_client: Arc<Client>,
) -> Result<String> {
	let api_url = api_base_url.to_string();

	let api_url = question_api_url(api_url);

	let client = http_client.clone();

	let mut headers = HeaderMap::new();

	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", api_key))?,
	);

	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

	let data = json!({
		 "model": model,
		 "messages": [{"role": "system", "content": "You are a helpful assistant."},{"role": "user", "content": text}]
	});

	trace!("{:?}", data);

	let res: Value = client
		.post(api_url)
		.headers(headers)
		.json(&data)
		.send()
		.await?
		.json()
		.await?;

	trace!("{:?}", res);

	let content = res["choices"][0]["message"]["content"].to_string();

	let content = content[1..content.len() - 1].to_string();

	Ok(content.replace("\\n", " \n "))
}

/// Constructs the URL for the OpenAI question API endpoint based on a given base URL.
///
/// # Arguments
///
/// * `api_url` - A `String` containing the base API URL supplied by the user.
///
/// # Returns
///
/// A `String` representing the complete URL to the OpenAI `chat/completions` endpoint.
///
/// # Logic
///
/// - If the provided `api_url` ends with `v1/`, the function appends `chat/completions` to it directly.
/// - If the provided `api_url` ends with `v1`, the function appends `/chat/completions` to it.
/// - If neither condition is met, the function appends `/v1/chat/completions` to the provided `api_url`.
///
/// # Examples
///
/// ```rust
/// let api_url1 = String::from("https://api.openai.com/v1/");
/// let api_url2 = String::from("https://api.openai.com/v1");
/// let api_url3 = String::from("https://api.openai.com");
///
/// assert_eq!(
///     question_api_url(api_url1),
///     "https://api.openai.com/v1/chat/completions"
/// );
/// assert_eq!(
///     question_api_url(api_url2),
///     "https://api.openai.com/v1/chat/completions"
/// );
/// assert_eq!(
///     question_api_url(api_url3),
///     "https://api.openai.com/v1/chat/completions"
/// );
/// ```
///
/// This function ensures the resulting URL always points to the `v1/chat/completions` endpoint,
/// minimizing user input errors in constructing the correct API URL.
pub fn question_api_url(api_url: String) -> String {
	if api_url.ends_with("v1/") {
		format!("{}chat/completions", api_url)
	} else if api_url.ends_with("v1") {
		format!("{}/chat/completions", api_url)
	} else {
		format!("{}/v1/chat/completions", api_url)
	}
}
