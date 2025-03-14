use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::{Config, DbConfig};
use crate::database::prelude::ServerImage;
use crate::database::server_image::Column;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::server::generate_image_pfp_server::load_localization_pfp_server_image;
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateAttachment,
	CreateInteractionResponseMessage,
};
use serenity::builder::CreateInteractionResponseFollowup;
use tracing::trace;
use uuid::Uuid;

pub struct GenerateImagePfPCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GenerateImagePfPCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for GenerateImagePfPCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		init(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn init(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	send_embed(ctx, command_interaction, "local", config.db.clone()).await
}

pub async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, image_type: &str,
	db_config: DbConfig,
) -> Result<()> {
	// Retrieve the guild ID from the command interaction
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized text for the server's profile picture image
	let pfp_server_image_localised_text =
		load_localization_pfp_server_image(guild_id.clone(), db_config.clone()).await?;

	// Create a deferred response to the command interaction
	let builder_message = Defer(CreateInteractionResponseMessage::new());

	// Send the deferred response
	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	// Retrieve the server's profile picture image
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

	let image = ServerImage::find()
		.filter(Column::ServerId.eq(guild_id.clone()))
		.filter(Column::ImageType.eq(image_type.to_string()))
		.one(&connection)
		.await?
		.ok_or(anyhow!(format!(
			"Server image with type {} not found",
			image_type
		)))?
		.image;

	// Decode the image from base64
	let input = image.trim_start_matches("data:image/png;base64,");

	let image_data: Vec<u8> = BASE64.decode(input)?;

	drop(image);

	// Generate a unique filename for the image
	let uuid = Uuid::new_v4();

	let image_path = format!("{}.png", uuid);

	// Construct the embed for the response
	let builder_embed = get_default_embed(None)
		.image(format!("attachment://{}", &image_path))
		.title(pfp_server_image_localised_text.title);

	// Create an attachment with the image
	let attachment = CreateAttachment::bytes(image_data, image_path);

	// Construct the follow-up response with the embed and the attachment
	let builder = CreateInteractionResponseFollowup::new()
		.embed(builder_embed)
		.files(vec![attachment]);

	// Send the follow-up response to the command interaction
	command_interaction
		.create_followup(&ctx.http, builder)
		.await?;

	trace!("Done");

	Ok(())
}
