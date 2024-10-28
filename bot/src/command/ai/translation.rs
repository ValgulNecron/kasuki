use std::sync::Arc;

use crate::command::ai::question::question_api_url;
use crate::command::command_trait::{Command, PremiumCommand, PremiumCommandType, SlashCommand};
use crate::config::Config;
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::{
	get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use crate::structure::message::ai::translation::load_localization_translation;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{multipart, Url};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
};
use tracing::trace;
use uuid::Uuid;

pub struct TranslationCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,

	pub command_name: String,
}

impl Command for TranslationCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for TranslationCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
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

		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
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

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let translation_localised = load_localization_translation(guild_id, config.db.clone()).await?;

	if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
		return Err(anyhow!("Unsupported file type"));
	}

	let builder_message = Defer(CreateInteractionResponseMessage::new());

	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	let allowed_extensions = ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm", "ogg"];

	let parsed_url = Url::parse(content.as_str())?;

	let path_segments = parsed_url
		.path_segments()
		.ok_or(anyhow!("Failed to get the path segments"))?;

	let last_segment = path_segments.last().unwrap_or_default();

	let file_extension = last_segment
		.rsplit('.')
		.next()
		.ok_or(anyhow!("Failed to get the file extension"))?
		.to_lowercase();

	if !allowed_extensions.contains(&&*file_extension) {
		return Err(anyhow!("Unsupported file extension"));
	}

	let response = reqwest::get(content.as_str()).await?; // save the file into a buffer
	let buffer = response.bytes().await?;

	let uuid_name = Uuid::new_v4().to_string();

	let client = reqwest::Client::new();

	let mut headers = HeaderMap::new();

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

	trace!("{} {}", token, model);

	let api_base_url = config
		.ai
		.transcription
		.ai_transcription_base_url
		.clone()
		.unwrap_or_default();

	// check the last 3 characters of the url if it v1/ or v1 or something else
	let api_base_url = if api_base_url.ends_with("v1/") {
		format!("{}audio/translations/", api_base_url)
	} else if api_base_url.ends_with("v1") {
		format!("{}/audio/translations/", api_base_url)
	} else {
		format!("{}/v1/audio/translations/", api_base_url)
	};

	trace!("{}", api_base_url);

	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
	);

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

	trace!("{}", res);

	let text = res["text"].as_str().unwrap_or("");

	trace!("{}", text);

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

		translation(lang, text.to_string(), api_key, api_base_url, model).await?
	} else {
		String::from(text)
	};

	let builder_embed = get_default_embed(None)
		.title(translation_localised.title)
		.description(text);

	let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

	command_interaction
		.create_followup(&ctx.http, builder_message)
		.await?;

	Ok(())
}

/// This asynchronous function runs the command interaction for transcribing an audio or video file.
///
/// It first retrieves the language and prompt for the transcription from the command interaction options.
/// It also retrieves the attachment to be transcribed.
///
/// It checks the content type of the attachment and returns an error if it is not an audio or video file.
///
/// It sends a deferred response to the command interaction.
///
/// It downloads the attachment and saves it to a local file.
///
/// It sends a request to the OpenAI API to transcribe the file.
///
/// It retrieves the transcription from the API response and sends a followup message with the transcription.
///
/// It handles any errors that occur during the process and returns an `AppError` if an error occurs.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.

pub async fn translation(
	lang: String, text: String, api_key: String, api_url: String, model: String,
) -> Result<String> {
	let prompt_gpt = format!("
            i will give you a text and a ISO-639-1 code and you will translate it in the corresponding language
            iso code: {}
            text:
            {}
            ", lang, text);

	let client = reqwest::Client::new();

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
