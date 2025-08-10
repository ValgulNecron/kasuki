//! This module defines the `GenerateImagePfPCommand` structure and related functionality
//! for handling a command interaction that generates server profile picture images.

use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandFiles, CommandType, EmbedContent, EmbedsContents};
use crate::database::prelude::ServerImage;
use crate::database::server_image::Column;
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::server::generate_image_pfp_server::load_localization_pfp_server_image;
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseMessage,
};
use uuid::Uuid;

/// The `GenerateImagePfPCommand` struct is used to encapsulate the necessary data
/// required to execute a command that generates a profile picture (PfP).
///
/// # Fields
///
/// * `ctx` - A `SerenityContext` instance that provides access to the Discord bot's
///           state and allows interacting with Discord APIs.
///
/// * `command_interaction` - A `CommandInteraction` instance that represents the
///                           interaction triggered by the user, containing information
///                           such as command arguments and user context.
///
/// This struct is typically used in scenarios where a Discord bot listens for a specific
/// user command to generate an image (e.g., a profile picture) and carries out the requested
/// functionality using the data available in these fields.
#[derive(Clone)]
pub struct GenerateImagePfPCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for GenerateImagePfPCommand,
	get_contents = |self_: GenerateImagePfPCommand| async move {
		self_.defer().await?;
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();
		let db_connection = bot_data.db_connection.clone();

		let embed_contents = get_content(ctx, command_interaction, "local", db_connection).await?;

		Ok(embed_contents)
	}
);

/// Asynchronously retrieves and processes the content for an image embed.
///
/// This function is used to fetch an image from the database for a specific server
/// (guild) and generate a corresponding embed content, including the localized title
/// and the image itself, ready to be sent as a response to a Discord command interaction.
///
/// # Arguments
///
/// * `ctx` - A reference to the `SerenityContext` used to interact with the Discord API.
/// * `command_interaction` - The `CommandInteraction` that triggered this function,
///   containing details about the guild, the command, and the user.
/// * `image_type` - A string slice specifying the type of image to retrieve (e.g.,
///   profile picture, banner, etc.).
/// * `db_config` - The `DbConfig` containing database connection details.
///
/// # Returns
///
/// * `Ok(EmbedContent<'static, 'static>)` - On success, returns the generated
///   `EmbedContent` which contains the localized title and the image as an attachment
///   for the embed.
/// * `Err` - Returns an error if any part of this process fails, such as database
///   retrieval, base64 decoding of the image, or response creation.
///
/// # Errors
///
/// This function can fail and return an error in the following cases:
/// 1. If the guild ID cannot be extracted from the command interaction.
/// 2. If the server's localized profile picture image text cannot be loaded from
///    localization.
/// 3. If creating a deferred response to the interaction fails.
/// 4. If connecting to the database results in an error.
/// 5. If the server image for the specified `image_type` does not exist in the database.
/// 6. If decoding the base64 image data fails.
/// 7. If constructing the embed content fails.
///
/// # Example
///
/// ```rust
/// let content = get_content(&ctx, &command_interaction, "profile_picture", db_config).await?;
///
/// match content {
///     Ok(embed_content) => {
///         // Send or use the embed content in your application
///     }
///     Err(err) => {
///         eprintln!("Error retrieving embed content: {:?}", err);
///     }
/// }
/// ```
pub async fn get_content<'a>(
	ctx: SerenityContext, command_interaction: CommandInteraction, image_type: &str,
	db_connection: Arc<DatabaseConnection>,
) -> Result<EmbedsContents<'a>> {
	// Retrieve the guild ID from the command interaction
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized text for the server's profile picture image
	let pfp_server_image_localised_text =
		load_localization_pfp_server_image(guild_id.clone(), db_connection.clone()).await?;

	let image = ServerImage::find()
		.filter(Column::ServerId.eq(guild_id.clone()))
		.filter(Column::ImageType.eq(image_type.to_string()))
		.one(&*db_connection)
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

	let embed_content = EmbedContent::new(pfp_server_image_localised_text.title)
		.images_url(format!("attachment://{}", image_path.clone()));
	let file = CommandFiles::new(image_path, image_data);
	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content])
		.add_file(file)
		.clone();

	Ok(embed_contents)
}
