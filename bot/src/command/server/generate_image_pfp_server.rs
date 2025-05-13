//! This module defines the `GenerateImagePfPCommand` structure and related functionality 
//! for handling a command interaction that generates server profile picture images.
use anyhow::{Result, anyhow};
use std::borrow::Cow;

use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedImage};
use crate::config::DbConfig;
use crate::database::prelude::ServerImage;
use crate::database::server_image::Column;
use crate::event_handler::BotData;
use crate::get_url;
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
pub struct GenerateImagePfPCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GenerateImagePfPCommand {
	/// Returns a reference to the `SerenityContext`.
	///
	/// # Description
	/// This method provides access to the `SerenityContext` associated with the current object.
	/// The `SerenityContext` is typically used within the Serenity framework for managing and interacting 
	/// with Discord bot-related state and events.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`).
	///
	/// # Example
	/// ```rust
	/// let context = my_object.get_ctx();
	/// // Use the context for interacting with Serenity functionality.
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance contained within the object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` associated with this object.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// println!("{:?}", command_interaction);
	/// ```
	///
	/// # Notes
	/// This function borrows the `CommandInteraction` immutably, ensuring that the returned reference 
	/// cannot be used to modify the underlying data.
	///
	/// # Usage
	/// This is commonly used to access the `CommandInteraction` for inspection or to retrieve details 
	/// relevant to a command execution process.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and compiles embed content for a given command interaction.
	///
	/// This function fetches the necessary data from the bot's context and configuration, 
	/// processes the command interaction to generate an `EmbedContent` object, and wraps 
	/// it in a `Vec` for ease of use.
	///
	/// # Returns
	///
	/// A `Result` containing a vector of `EmbedContent` instances if the operation is 
	/// successful, or an error if something goes wrong during the process.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - Retrieving the context or configuration fails.
	/// - Processing the command interaction fails while generating the embed content.
	///
	/// # Example
	///
	/// ```rust
	/// let contents = my_instance.get_contents().await?;
	/// ```
	///
	/// # Dependencies
	///
	/// - `ctx`: Used for accessing the application context, including shared data.
	/// - `bot_data`: Acquired from the context, containing the bot's configuration and database.
	/// - `get_content`: An asynchronous function responsible for processing the interaction and 
	///   generating embed content.
	///
	/// # Notes
	///
	/// - The function assumes the presence of `BotData` in the shared context and that `config.db` 
	///   is properly set up to support the `get_content` function.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let embed_content = get_content(ctx, command_interaction, "local", config.db.clone()).await?;
		
		Ok(vec![embed_content])	
	}
}

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
	ctx: &'a SerenityContext, command_interaction: &CommandInteraction, image_type: &str,
	db_config: DbConfig,
) -> Result<EmbedContent<'static, 'static>> {
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

	Ok(
		EmbedContent::new(pfp_server_image_localised_text.title).images(Some(vec![EmbedImage {
			attachment: CreateAttachment::bytes(image_data, Cow::from(image_path.clone())),
			image: image_path,
		}])),
	)
}
