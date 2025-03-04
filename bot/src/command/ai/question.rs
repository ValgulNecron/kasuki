use crate::command::command_trait::{
	Command, Embed, EmbedContent, EmbedType, PremiumCommand, PremiumCommandType, SlashCommand,
};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use anyhow::{Result, anyhow};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::trace;
pub struct QuestionCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
	pub command_name: String,
}

impl Command for QuestionCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for QuestionCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = &self.ctx;
		let command_interaction = &self.command_interaction;
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

		let text = question(prompt, api_key, api_base_url, model).await?;

		let embed_content = EmbedContent {
			title: String::new(),
			description: text,
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

async fn question(
	text: &String, api_key: String, api_base_url: String, model: String,
) -> Result<String> {
	let api_url = api_base_url.to_string();

	let api_url = question_api_url(api_url);

	let client = reqwest::Client::new();

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

pub fn question_api_url(api_url: String) -> String {
	if api_url.ends_with("v1/") {
		format!("{}chat/completions", api_url)
	} else if api_url.ends_with("v1") {
		format!("{}/chat/completions", api_url)
	} else {
		format!("{}/v1/chat/completions", api_url)
	}
}
