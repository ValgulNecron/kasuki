//! Documentation for `TranscriptCommand` implementation and associated functionality.
use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::{
	get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;
use uuid::Uuid;

#[slash_command(
	name = "transcript", desc = "Generate a transcript from a video.",
	command_type = SubCommand(parent = "ai"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	extra_fields = [command_name: String = full_command_name.to_string()],
	args = [
		(name = "video", desc = "Upload video file (max. 25MB).", arg_type = Attachment, required = true, autocomplete = false),
		(name = "prompt", desc = "A guide text for audio style. Must match the audio language.", arg_type = String, required = false, autocomplete = false),
		(name = "lang", desc = "Select input language (ISO-639-1)", arg_type = String, required = false, autocomplete = false)
	],
)]
async fn transcript_command(self_: TranscriptCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let command_interaction = self_.get_command_interaction();
	let bot_data = ctx.data::<BotData>().clone();
	let config = bot_data.config.clone();

	if self_
		.check_hourly_limit(
			self_.command_name.clone(),
			&bot_data,
			PremiumCommandType::AITranscript,
		)
		.await?
	{
		return Err(anyhow!(
			"You have reached your hourly limit. Please try again later.",
		));
	}


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
	let db_connection = bot_data.db_connection.clone();

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

	let client = bot_data.http_client.clone();
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

	let lang = get_guild_language(guild_id, db_connection).await;
	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
	let title = USABLE_LOCALES.lookup(&lang_id, "ai_transcript-title");

	let embed_content = EmbedContent::new(title).description(text.to_string());

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
