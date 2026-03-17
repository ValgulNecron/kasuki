use crate::command::context::CommandContext;
use crate::command::embed_content::{CommandFiles, EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::{
	get_option_map_integer_subcommand, get_option_map_string_subcommand,
};
use anyhow::anyhow;
use fluent_templates::Loader;
use image::EncodableLayout;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use shared::service::ai;
use tracing::error;
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
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	if self_
		.check_hourly_limit(
			self_.command_name.clone(),
			&cx.bot_data,
			PremiumCommandType::AIImage,
		)
		.await?
	{
		return Err(anyhow!(
			"You have reached your hourly limit. Please try again later.",
		));
	}

	let config = &cx.config;
	let int_map = get_option_map_integer_subcommand(&cx.command_interaction);
	let str_map = get_option_map_string_subcommand(&cx.command_interaction);
	let client = &cx.http_client;
	let image_store = &cx.image_store;

	let n = *int_map.get("n").unwrap_or(&1);

	let prompt = str_map
		.get("description")
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);
	let model = config.ai.image.ai_image_model.as_deref().unwrap_or_default();
	let quality = config.ai.image.ai_image_quality.as_deref();
	let style = config.ai.image.ai_image_style.as_deref();
	let size = config.ai.image.ai_image_size.as_deref().unwrap_or("1024x1024");

	let data = ai::build_image_payload(prompt, n, model, quality, style, size);

	let uuid_name = Uuid::new_v4();
	let filename = format!("{}.png", uuid_name);
	let token = config.ai.image.ai_image_token.as_deref().unwrap_or_default();
	let url = config.ai.image.ai_image_base_url.as_deref().unwrap_or_default();
	let url = ai::normalize_api_url(url, "images/generations");

	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert(
		reqwest::header::AUTHORIZATION,
		reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))?,
	);
	headers.insert(
		reqwest::header::CONTENT_TYPE,
		reqwest::header::HeaderValue::from_static("application/json"),
	);

	let res = client.post(&url).headers(headers).json(&data).send().await?;
	let res = res.json().await?;

	let user_id = cx.command_interaction.user.id.to_string();

	let bytes = ai::download_images_from_response(
		res,
		&user_id,
		&cx.guild_id,
		image_store,
		client,
	)
	.await?;

	let lang_id = cx.lang_id().await;
	let title = USABLE_LOCALES.lookup(&lang_id, "ai_image-title");

	let embed_content = EmbedContent::new(title).description(String::new());

	let mut embed_contents = vec![];
	let mut command_files = vec![];

	if n == 1 {
		let attachment = bytes[0].as_bytes().to_vec();
		let name = filename;

		command_files.push(CommandFiles::new(name.clone(), attachment));
		embed_contents.push(
			embed_content
				.clone()
				.images_url(format!("attachment://{}", name.clone())),
		);
	} else {
		let attachments: Vec<(Vec<u8>, String)> = bytes
			.iter()
			.enumerate()
			.map(|(index, byte)| {
				let name = format!("{}_{}.png", filename, index);
				let byte = byte.as_bytes().to_vec();
				(byte, format!("{}_{}.png", name, index))
			})
			.collect();

		for attachement in attachments {
			let name = attachement.1;
			let bytes = attachement.0;
			command_files.push(CommandFiles::new(name.clone(), bytes.clone()));
			embed_contents.push(
				embed_content
					.clone()
					.images_url(format!("attachment://{}", name.clone())),
			);

			let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
			let storage_key =
				format!("ai_images/ai_{}_{}_{}.png", user_id, cx.guild_id, timestamp);
			if let Err(e) = image_store.save(&storage_key, &bytes).await {
				error!("Error saving AI image: {}", e);
			}
		}
	};

	let embed_contents = EmbedsContents::new(embed_contents).add_files(command_files);

	Ok(embed_contents)
}
