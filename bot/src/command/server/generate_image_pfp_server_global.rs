//! The `GenerateGlobalImagePfPCommand` struct is responsible for handling
//! a command interaction that generates a global profile picture representation
//! for a user. This struct implements the `Command` trait, which defines methods
//! for managing the context, interaction, and generating command content.
//!
//! # Fields
//! - `ctx`:
//!   The `SerenityContext` that provides access to information and utilities
//!   needed to handle the command interaction.
//! - `command_interaction`:
//!   The `CommandInteraction` object containing details about the interaction
//!   triggered by the user.
//!
//! # Responsibilities
//! - Provides access to the context and command interaction.
//! - Retrieves data required to embed content related to the "global profile picture" command.
//! - Executes the command to generate global profile picture content.
//!
//! # Trait `Command` Implementation
//!
//! ## `get_ctx`
//! Returns a reference to the Serenity context associated with this command.
//!
//! ## `get_command_interaction`
//! Returns a reference to the command interaction that triggered this execution.
//!
//! ## `get_contents`
//! Asynchronously generates the command content as a vector of `EmbedContent` objects.
//!
//! This method:
//! 1. Retrieves necessary data, including bot configuration and database connection, from the context.
//! 2. Interacts with the profile picture generation logic using the helper function `get_content`.
//! 3. Returns the generated content as a result.
//!
//! # Errors
//! The `get_contents` method can return an error if there are issues in fetching the embed content,
//! such as communication or configuration problems.
//!
//! # Example Usage
//! This code is typically used as part of an event handler where the bot processes user
//! interactions and responds with the appropriate image or embed content. The structure
//! contributes to handling the `"global"` image generation command.
use crate::command::command::Command;
use crate::command::embed_content::EmbedsContents;
use crate::command::server::generate_image_pfp_server::get_content;
use crate::event_handler::BotData;
use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing the `GenerateGlobalImagePfPCommand`, which encapsulates
/// the context and interaction data necessary to process a "Generate Global Image Profile Picture" command.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext` which provides access to Discord API functionality,
///           including data caches and utilities for interacting with the Discord bot.
/// * `command_interaction` - The `CommandInteraction` object containing information
///                            about the specific command interaction from the user,
///                            such as command arguments, user details, and the originating channel.
pub struct GenerateGlobalImagePfPCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GenerateGlobalImagePfPCommand {
	/// Returns a reference to the `SerenityContext` associated with this instance.
	///
	/// # Returns
	///
	/// * `&SerenityContext` - A reference to the current `SerenityContext`.
	///
	/// # Examples
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the context object as needed
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Returns a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// This method provides access to the `command_interaction` field, which contains
	/// the details of the command interaction linked to the object. It can be used
	/// to access specific data or perform operations associated with the command.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` instance.
	///
	/// # Example
	/// ```
	/// let command_interaction = object.get_command_interaction();
	/// // Use the command_interaction as needed.
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves a vector of `EmbedContent` elements.
	///
	/// This function fetches the necessary context, bot data, and configuration,
	/// and then retrieves a single `EmbedContent` instance based on a global scope
	/// using the asynchronous helper function `get_content`. The resulting
	/// `EmbedContent` is returned inside a vector.
	///
	/// # Returns
	///
	/// * `Ok(Vec<EmbedContent<'_, '_>>)` - A vector containing the fetched `EmbedContent`.
	/// * `Err` - An error if the operation fails.
	///
	/// # Errors
	///
	/// This function will return an error if:
	/// - The bot's context or configuration cannot be accessed.
	/// - The `get_content` function encounters an issue during its execution.
	///
	/// # Usage
	///
	/// ```rust
	/// let contents = some_instance.get_contents().await?;
	/// for content in contents {
	///     // Process each content
	/// }
	/// ```
	///
	/// # Dependencies
	/// - This function depends on the presence of a proper context, bot data,
	///   and a valid database connection in the configuration.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let embed_contents =
			get_content(ctx, command_interaction, "global", config.db.clone()).await?;

		Ok(embed_contents)
	}
}
