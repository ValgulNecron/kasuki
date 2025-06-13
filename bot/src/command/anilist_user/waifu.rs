//! WaifuCommand Struct
//!
//! A command used to fetch details about a character from AniList and return it as embed content.
//!
//! ## Fields
//!
//! - `ctx`: The Serenity `Context` allowing access to bot and framework state.
//! - `command_interaction`: The `CommandInteraction` containing data related to the Discord command input.
//!
//! ## Example Usage
//! ```no_run
//! use serenity::all::CommandInteraction;
//! use serenity::all::Context as SerenityContext;
//!
//! let command = WaifuCommand {
//!     ctx: serenity_context,
//!     command_interaction: command_interaction
//! };
//!
//! command.get_contents().await?;
//! ```
//!
use anyhow::Result;

use serenity::all::{CommandInteraction, Context as SerenityContext};

use crate::command::anilist_user::character::get_character_by_id;
use crate::command::command::Command;
use crate::command::embed_content::EmbedsContents;
use crate::event_handler::BotData;
use crate::structure::run::anilist::character::character_content;

/// The `WaifuCommand` struct represents a wrapper for handling a waifu-related command in a Discord bot using the Serenity library.
///
/// This struct contains the context of the bot and the interaction data for the command, allowing it to process and respond accordingly.
///
/// # Fields
///
/// * `ctx` - The [`SerenityContext`] that provides access to the Discord API, bot state, and interaction-related utilities.
/// * `command_interaction` - The [`CommandInteraction`] containing the details of the command issued by a user, such as command arguments and user details.
///
/// # Example
///
/// ```rust
/// use my_bot::WaifuCommand;
/// use serenity::prelude::*;
/// use serenity::model::prelude::*;
///
/// let waifu_command = WaifuCommand {
///     ctx,
///     command_interaction,
/// };
/// // Handle command logic here...
/// ```
///
/// This struct is part of the command handling mechanism for processing Discord interactions and generating responses. It is used within async command handlers or function calls to process user input.
///
/// # Notes
/// Both `SerenityContext` and `CommandInteraction` are required for this struct to function effectively in the context of a Discord bot.
pub struct WaifuCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for WaifuCommand {
	/// Returns a reference to the `SerenityContext` associated with the current instance.
	///
	/// This method provides access to the `SerenityContext`, which holds the state and framework
	/// context of the Serenity bot, allowing interaction with Discord's API and various bot-related data.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext`.
	///
	/// # Example
	/// ```
	/// let context = instance.get_ctx();
	/// // Use `context` to interact with the bot's framework state.
	/// ```
	///
	/// # Note
	/// This method does not take ownership of the context, it only provides a shared reference.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with this instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` stored within the object.
	///
	/// # Example
	/// ```
	/// let interaction = object.get_command_interaction();
	/// // Use the retrieved `CommandInteraction`
	/// ```
	///
	/// This method is useful for accessing the encapsulated `CommandInteraction` when
	/// you only need to read or utilize its data without modifying it.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronous function to retrieve and generate embed content for a specified character from AniList.
	///
	/// # Description
	/// This function fetches the character data from AniList using a hardcoded character ID (`156323`)
	/// and processes it to generate content that can be used as embed responses in a Discord bot context.
	///
	/// # Steps:
	/// 1. Retrieves the bot's context and access to shared dependencies like configuration and cache.
	/// 2. Fetches the character data from AniList using the provided character ID.
	/// 3. Generates embed content for the fetched character data, utilizing the bot's configuration and interaction details.
	///
	/// # Returns
	/// If successful, returns a `Result` containing a `Vec` of `EmbedContent` objects
	/// that can be utilized for embed messages. If an error occurs during data retrieval
	/// or processing, it returns the corresponding error.
	///
	/// # Errors
	/// The function will return an error if:
	/// - The AniList character data could not be fetched (`get_character_by_id` fails).
	/// - The embed content generation process fails (`character_content` fails).
	///
	/// # Dependencies
	/// - `get_character_by_id`: An asynchronous function that fetches character data from AniList using the provided character ID.
	/// - `character_content`: An asynchronous function that processes the fetched character data and interaction details to generate embed content.
	///
	/// # Example
	/// ```rust
	/// let embed_contents = self.get_contents().await?;
	/// for embed in embed_contents {
	///     send_embed(channel, embed).await?;
	/// }
	/// ```
	///
	/// # Returns
	/// `Result<Vec<EmbedContent>, Error>`: The formatted embed content or an error if retrieval or processing fails.
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let config = bot_data.config.clone();
		let db_config = config.db.clone();

		let anilist_cache = bot_data.anilist_cache.clone();

		// Execute the corresponding search function based on the specified type
		// Fetch the data of the character with ID 156323 from AniList
		let value = 156323;

		let data = get_character_by_id(value, anilist_cache).await?;

		let embed_content = character_content(command_interaction, data, db_config).await?;

		Ok(embed_content)
	}
}
