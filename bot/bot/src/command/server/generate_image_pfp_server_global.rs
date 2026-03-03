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
use crate::command::command::CommandRun;
use crate::command::server::generate_image_pfp_server::get_content;
use crate::event_handler::BotData;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[slash_command(
	name = "guild_image_g", desc = "Generate global profile picture for the guild.",
	command_type = SubCommand(parent = "server"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn generate_global_image_pfp_command(
	self_: GenerateGlobalImagePfPCommand,
) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx().clone();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction().clone();
	let db_connection = bot_data.db_connection.clone();

	let embed_contents = get_content(ctx, command_interaction, "global", db_connection).await?;

	Ok(embed_contents)
}
