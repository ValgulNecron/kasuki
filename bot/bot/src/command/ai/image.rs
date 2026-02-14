use bytes::Bytes;
use std::sync::Arc;

use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandFiles, CommandType, EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::{
	get_option_map_integer_subcommand, get_option_map_string_subcommand,
};
use crate::helper::image_saver::general_image_saver::image_saver;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use image::EncodableLayout;
use kasuki_macros::slash_command;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::config::Config;
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use tracing::{error, trace};
use unic_langid::LanguageIdentifier;
use uuid::Uuid;

#[slash_command(
	name = "image", desc = "Generate an image.",
	command_type = SubCommand(parent = "ai"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	extra_fields = [command_name: String = full_command_name.to_string()],
	args = [
		(name = "description", desc = "Enter a description of the image you want to generate.", arg_type = String, required = true, autocomplete = false),
		(name = "n", desc = "Number of images to generate.", arg_type = Integer, required = false, autocomplete = false)
	],
)]
async fn image_command(self_: ImageCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	if self_
		.check_hourly_limit(
			self_.command_name.clone(),
			&bot_data.clone(),
			PremiumCommandType::AIImage,
		)
		.await?
	{
		return Err(anyhow!(
			"You have reached your hourly limit. Please try again later.",
		));
	}

	let command_interaction = self_.get_command_interaction();
	let config = bot_data.config.clone();
	let map = get_option_map_integer_subcommand(command_interaction);
	let client = bot_data.http_client.clone();

	let n = *map.get(&String::from("n")).unwrap_or(&1);
	let data = get_value(command_interaction, n, &config);

	self_.defer().await?;

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

	let lang = get_guild_language(guild_id.clone(), bot_data.db_connection.clone()).await;
	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
	let title = USABLE_LOCALES.lookup(&lang_id, "ai_image-title");

	let embed_content = EmbedContent::new(title).description(String::new());

	let mut embed_contents = vec![];
	let mut command_files = vec![];

	if n == 1 {
		let attachment = image_with_n_equal_1(bytes.clone()).await;
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
			command_files.push(CommandFiles::new(name.clone(), bytes.clone()));
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

async fn image_with_n_equal_1(bytes: Vec<Bytes>) -> Vec<u8> {
	let bytes = bytes[0].as_bytes().to_vec();

	bytes
}

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
			error!("Failed to deserialize response into Root: {}", e);
			error!("Raw response body: {}", json);
			let root1: Root1 = serde_json::from_value(json)?;

			return Err(anyhow!(format!(
				"Error: {} ............ {:?}",
				e, root1.error
			)));
		},
	};

	let urls: Vec<String> = root
		.data
		.iter()
		.filter_map(|data| data.url.clone())
		.collect();

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

#[derive(Debug, Deserialize)]

struct Root {
	#[serde(rename = "data")]
	data: Vec<Data>,
}

#[derive(Debug, Deserialize)]

struct Data {
	url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]

struct AiError {
	pub message: String,
	#[serde(rename = "type")]
	pub error_type: String,
	pub param: Option<String>,
	pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]

struct Root1 {
	pub error: AiError,
}
