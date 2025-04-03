use crate::command::command_trait::{
	Command, Embed, EmbedContent, EmbedImage, EmbedType, SlashCommand,
};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime::random_image::load_localization_random_image;
use anyhow::{Result, anyhow};
use image::EncodableLayout;
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateAttachment};
use uuid::Uuid;

pub struct AnimeRandomImageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AnimeRandomImageCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for AnimeRandomImageCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.command_interaction.clone();
		let config = bot_data.config.clone();

		// Retrieve the type of image to fetch from the command interaction
		let map = get_option_map_string_subcommand(&command_interaction);

		let image_type = map
			.get(&String::from("image_type"))
			.ok_or(anyhow!("No image type specified"))?;

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let random_image_localised =
			load_localization_random_image(guild_id, config.db.clone()).await?;

		self.defer().await?;

		send_embed(
			ctx,
			&command_interaction,
			image_type,
			random_image_localised.title,
			"sfw",
			self,
		)
		.await
	}
}

pub async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, image_type: &String,
	title: String, endpoint: &str, self_: &impl Command,
) -> Result<()> {
	// Construct the URL to fetch the image from
	let url = format!("https://api.waifu.pics/{}/{}", endpoint, image_type);

	// Fetch the image from the URL
	let resp = reqwest::get(&url).await?;

	// Parse the response as JSON
	let json: serde_json::Value = resp.json().await?;

	// Retrieve the URL of the image from the JSON
	let image_url = json["url"]
		.as_str()
		.ok_or(anyhow!("No image found"))?
		.to_string();

	// Fetch the image from the image URL
	let response = reqwest::get(image_url).await?;

	// Retrieve the bytes of the image from the response
	let bytes = response.bytes().await?;

	// Generate a UUID for the filename of the image
	let uuid_name = Uuid::new_v4();

	let filename = format!("{}.gif", uuid_name);

	// Construct the attachment for the image
	let bytes = bytes.as_bytes().to_vec();
	let attachment = CreateAttachment::bytes(bytes, filename.clone());

	let content = EmbedContent {
		title,
		description: "".to_string(),
		thumbnail: None,
		url: None,
		command_type: EmbedType::Followup,
		colour: None,
		fields: vec![],
		images: Some(vec![EmbedImage {
			attachment,
			image: filename,
		}]),
		action_row: None,
		images_url: None,
	};

	self_.send_embed(content).await
}
