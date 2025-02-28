use std::sync::Arc;

use crate::command::command_trait::{
	Command, Embed, EmbedContent, EmbedImage, EmbedType, PremiumCommand, PremiumCommandType,
	SlashCommand,
};
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
use prost::Message;
use prost::bytes::Bytes;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateAttachment};
use tracing::{error, trace};
use uuid::Uuid;

pub struct ImageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
	pub command_name: String,
}

impl Command for ImageCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for ImageCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = &self.ctx;
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

		let command_interaction = &self.command_interaction;

		let config = bot_data.config.clone();

		let map = get_option_map_integer_subcommand(command_interaction);

		let n = *map.get(&String::from("n")).unwrap_or(&1);

		let data = get_value(command_interaction, n, &config);

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let image_localised = load_localization_image(guild_id.clone(), config.db.clone()).await?;

		self.defer().await?;

		let uuid_name = Uuid::new_v4();

		let filename = format!("{}.png", uuid_name);

		let token = config.ai.image.ai_image_token.clone().unwrap_or_default();

		let url = config
			.ai
			.image
			.ai_image_base_url
			.clone()
			.unwrap_or_default();

		// check the last 3 characters of the url if it v1/ or v1 or something else
		let url = if url.ends_with("v1/") {
			format!("{}images/generations", url)
		} else if url.ends_with("v1") {
			format!("{}/images/generations", url)
		} else {
			format!("{}/v1/images/generations", url)
		};

		let client = reqwest::Client::new();

		let token = token.as_str();

		let mut headers = HeaderMap::new();

		headers.insert(
			AUTHORIZATION,
			HeaderValue::from_str(&format!("Bearer {}", token))?,
		);

		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

		let url = url.as_str();

		let res = client.post(url).headers(headers).json(&data).send().await?;

		let res = res.json().await?;

		trace!(?res);

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

		let images = if n == 1 {
			let attachment = image_with_n_equal_1(filename.clone(), bytes.clone()).await;
			let name = format!("attachment://{}", &filename);
			vec![EmbedImage {
				attachment,
				image: name,
			}]
		} else {
			let (images, filenames) = image_with_n_greater_than_1(filename, bytes).await;
			images
				.into_iter()
				.zip(filenames.into_iter())
				.map(|(attachment, filename)| EmbedImage {
					attachment,
					image: filename,
				})
				.collect()
		};

		for image in images.clone() {
			let bytes = image.attachment.data.clone();
			let filename = image.attachment.filename.clone();
			let image_config = bot_data.config.image.clone();
			let bytes: Vec<u8> = bytes.clone().to_owned().into();
			image_saver(
				guild_id.to_string(),
				filename.to_string(),
				bytes,
				image_config.save_server.unwrap_or_default(),
				image_config.token.unwrap_or_default(),
				image_config.save_image,
			)
			.await?;
		}

		let embed_content = EmbedContent {
			title: image_localised.title,
			description: String::new(),
			thumbnail: None,
			url: None,
			command_type: EmbedType::Followup,
			colour: None,
			fields: vec![],
			images: Some(images),
			action_row: None,
			images_url: None,
		};

		self.send_embed(embed_content).await
	}
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

async fn image_with_n_equal_1<'a>(filename: String, bytes: Vec<Bytes>) -> CreateAttachment<'a> {
	let bytes = bytes[0].as_bytes().to_vec();
	let attachment = CreateAttachment::bytes(bytes, filename);

	attachment
}

async fn image_with_n_greater_than_1<'a>(
	filename: String, bytes: Vec<Bytes>,
) -> (Vec<CreateAttachment<'a>>, Vec<String>) {
	let attachments: (Vec<CreateAttachment>, Vec<String>) = bytes
		.iter()
		.enumerate()
		.map(|(index, byte)| {
			let filename = format!("{}_{}.png", filename, index);
			let byte = byte.as_bytes().to_vec();
			(
				CreateAttachment::bytes(byte, filename.clone()),
				format!("{}_{}.png", filename, index),
			)
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
			body.clone().encode_to_vec(),
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
	url: String,
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
