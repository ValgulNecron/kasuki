//! Documentation for `TranscriptCommand` implementation and associated functionality.
use crate::command::command_trait::{
	Command, CommandRun, EmbedContent, EmbedType, PremiumCommand, PremiumCommandType,
};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::{
	get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use crate::structure::message::ai::transcript::load_localization_transcript;
use anyhow::{Result, anyhow};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Url, multipart};
use serde_json::Value;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::trace;
use uuid::Uuid;
/// Structure representing a transcript command in a Discord bot.
///
/// A `TranscriptCommand` encapsulates the details needed to process and execute
/// a specific bot command in the context of a Discord interaction.
///
/// # Fields
///
/// * `ctx` - Represents the context of the bot's interaction with Discord, providing
///   information and utilities necessary for managing and interacting with the bot.
///   This is usually an instance of `SerenityContext`, which facilitates the connection
///   with the Discord API and bot's state.
///
/// * `command_interaction` - Represents the interaction data triggered by the user for
///   the given command. This includes all information regarding the interaction,
///   such as user inputs, source channel, and other relevant metadata. Typically an
///   instance of the `CommandInteraction` type.
///
/// * `command_name` - A `String` representing the name of the command invoked by the
///   user. This is used to identify which command was executed and respond accordingly.
///
/// # Example Usage
///
/// This struct is primarily used to manage and execute commands received by the bot.
/// It is passed to functions that process bot commands and perform the desired actions.
///
/// ```rust
/// let command = TranscriptCommand {
///     ctx,
///     command_interaction,
///     command_name: "example_command".to_string(),
/// };
///
/// // Use the `command` struct to process the interaction.
/// process_command(command);
/// ```
pub struct TranscriptCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
	pub command_name: String,
}

impl Command for TranscriptCommand {
	/// Returns a reference to the `SerenityContext`.
	///
	/// This method provides access to the `SerenityContext` associated with the current instance,
	/// allowing interaction with Discord's API and gateway functionality.
	///
	/// # Returns
	/// A reference to the `SerenityContext`.
	///
	/// # Example
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use ctx to interact with the Discord API.
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` field of the object.
	///
	/// # Example
	/// ```rust
	/// let interaction = object.get_command_interaction();
	/// // Use the `interaction` reference as needed
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves the content of an audio or video file, processes it using a transcription API,
	/// and returns the transcribed text as an embed response.
	///
	/// # Errors
	///
	/// This function returns an `Err` in the following cases:
	/// - The user exceeds their hourly limit for the AI transcription command.
	/// - No video attachment is provided or the content type for the attachment is unsupported.
	/// - The file extension of the attachment is not in the list of allowed extensions.
	/// - A failure occurs while parsing the URL or download request for the attached file.
	/// - The transcription API configuration is invalid or the API request fails.
	/// - An error occurs while parsing the API response.
	///
	/// # Workflow
	/// 1. Verifies if the user has exceeded their hourly command limit.
	/// 2. Parses the input options, retrieves the language prompt, and validates the file attachment.
	/// 3. Ensures the file type and extension are supported for transcription.
	/// 4. Downloads the file from the provided URL and processes it using an external transcription API.
	/// 5. Returns the transcribed text in an embed format.
	///
	/// # Returns
	/// On success, returns a `Result<Vec<EmbedContent<'_, '_>>>` containing the embed with the transcribed content.
	///
	/// # Configuration Requirements
	/// - The transcription API settings (base URL, token, and model) should be configured in the `BotData` object.
	/// - Expected valid audio/video file types include: `mp3`, `mp4`, `mpeg`, `mpga`, `m4a`, `wav`, `webm`, `ogg`.
	///
	/// # Parameters
	/// This function operates on a struct instance, requiring the following context:
	/// - `self`: Includes methods for obtaining the command interaction and configuration context.
	///
	/// # Example
	/// ```rust
	/// let contents = instance.get_contents().await?;
	/// for content in contents {
	///     println!("Embed Description: {}", content.description());
	/// }
	/// ```
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();

		if self
			.check_hourly_limit(
				self.command_name.clone(),
				&bot_data,
				PremiumCommandType::AITranscript,
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
		let prompt = map
			.get(&String::from("lang"))
			.unwrap_or(DEFAULT_STRING)
			.clone();
		let lang = map
			.get(&String::from("prompt"))
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

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let transcript_localised =
			load_localization_transcript(guild_id, config.db.clone()).await?;


		let response = reqwest::get(content.to_string()).await?;
		let buffer = response.bytes().await?;
		let uuid_name = Uuid::new_v4().to_string();

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
		let url = if api_base_url.ends_with("v1/") {
			format!("{}audio/transcriptions/", api_base_url)
		} else if api_base_url.ends_with("v1") {
			format!("{}/audio/transcriptions/", api_base_url)
		} else {
			format!("{}/v1/audio/transcriptions/", api_base_url)
		};

		let client = reqwest::Client::new();
		let mut headers = HeaderMap::new();
		headers.insert(
			AUTHORIZATION,
			HeaderValue::from_str(&format!("Bearer {}", token))?,
		);

		let part = multipart::Part::bytes(buffer.to_vec())
			.file_name(uuid_name)
			.mime_str(content_type.as_str())?;
		let form = multipart::Form::new()
			.part("file", part)
			.text("model", model)
			.text("prompt", prompt)
			.text("language", lang)
			.text("response_format", "json");

		let response_result = client
			.post(url)
			.headers(headers)
			.multipart(form)
			.send()
			.await;
		let response = response_result?;

		let res_result: Result<Value, reqwest::Error> = response.json().await;
		let res = res_result?;
		let text = res["text"].as_str().unwrap_or("");
		let embed_content = EmbedContent::new(transcript_localised.title)
			.description(text.to_string())
			.command_type(EmbedType::Followup);

		Ok(vec![embed_content])
	}
}