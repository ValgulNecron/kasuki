use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::command::context::CommandContext;
use crate::command::embed_content::{CommandFiles, EmbedContent, EmbedsContents};
use kasuki_macros::slash_command;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::prelude::ServerImage;
use shared::database::server_image::Column;
use shared::image_saver::storage::ImageStore;
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use uuid::Uuid;

#[slash_command(
	name = "guild_image", desc = "Generate profile picture for the guild.",
	command_type = SubCommand(parent = "server"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn generate_image_pfp_command(self_: GenerateImagePfPCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let embed_contents =
		get_content(cx.ctx.clone(), cx.command_interaction.clone(), "local", cx.db.clone(), &cx.image_store).await?;

	Ok(embed_contents)
}

pub async fn get_content<'a>(
	_ctx: SerenityContext, command_interaction: CommandInteraction, image_type: &str,
	db_connection: Arc<DatabaseConnection>, image_store: &Arc<dyn ImageStore>,
) -> Result<EmbedsContents<'a>> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let lang_id = get_language_identifier(guild_id.clone(), db_connection.clone()).await;

	let image_key = ServerImage::find()
		.filter(Column::ServerId.eq(guild_id.clone()))
		.filter(Column::ImageType.eq(image_type.to_string()))
		.one(&*db_connection)
		.await?
		.ok_or(anyhow!(format!(
			"Server image with type {} not found",
			image_type
		)))?
		.image;

	let image_data = image_store
		.load(&image_key)
		.await
		.map_err(|_| anyhow!("Failed to load server image from storage: {}", image_key))?;

	let uuid = Uuid::new_v4();
	let image_path = format!("{}.png", uuid);

	let embed_content = EmbedContent::new(
		USABLE_LOCALES.lookup(&lang_id, "server_generate_image_pfp_server-title"),
	)
	.images_url(format!("attachment://{}", image_path.clone()));
	let file = CommandFiles::new(image_path, image_data);
	let embed_contents = EmbedsContents::new(vec![embed_content])
		.add_files(vec![file]);

	Ok(embed_contents)
}
