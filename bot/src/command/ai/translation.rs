use crate::command::ai::question::question_api_url;
use crate::command::command_trait::EmbedContent;
use crate::command::command_trait::{
	Command, Embed, EmbedType, PremiumCommand, PremiumCommandType, SlashCommand,
};
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
		let ctx = &self.ctx;
		let command_interaction = &self.command_interaction;
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

		let translation_localised =
			load_localization_translation(guild_id, config.db.clone()).await?;

		if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
			return Err(anyhow!("Unsupported file type"));
		}

		self.defer().await?;

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
		let embed_content = EmbedContent {
			title: translation_localised.title,
			description: text.to_string(),
			thumbnail: None,
			url: None,
			command_type: EmbedType::Followup,
			colour: None,
			fields: vec![],
			images: None,
			action_row: None,
			images_url: None,
		};
		self.send_embed(embed_content).await
	}
}
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
