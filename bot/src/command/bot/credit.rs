//! The `CreditCommand` struct and its associated implementation represent a command
//! that retrieves and displays localized credit information in the form of embed content.
//!
//! ## Struct: `CreditCommand`
//!
//! ### Fields:
//! - `ctx`:
//!     The `SerenityContext` that provides access to the bot's state and surroundings.
//! - `command_interaction`:
//!     The `CommandInteraction` object that encapsulates details about the slash command interaction.
//!
//! ## Implementations:
//!
//! ### Trait: `Command`
//!
//! This struct implements the `Command` trait, providing the following methods:
//!
//! - `fn get_ctx(&self) -> &SerenityContext`
//!     - Returns a reference to the bot's context object (`SerenityContext`).
//!     - Useful for accessing bot state, cache, or data shared across handlers.
//!
//! - `fn get_command_interaction(&self) -> &CommandInteraction`
//!     - Retrieves the slash command interaction that triggered the command.
//!     - Provides interaction-specific details such as the guild, channel, or user.
//!
//! - `async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>>`
//!     - Generates embed content describing the credit information.
//!     - Uses the bot's shared data to load localized credit details based on the guild ID associated with the command interaction.
//!     - Localized credit data (`credit_localised`) is retrieved asynchronously using `load_localization_credit`.
//!     - Processes the credits to concatenate their descriptions into a single string.
//!     - Builds and returns a list of `EmbedContent` objects with a descriptive title and content.
//!
//! ### Asynchronous Process:
//! 1. Accesses the bot's shared `BotData`, which includes the bot's configuration.
//! 2. Determines the guild ID from the slash command interaction, defaulting to `"0"` if none exists (e.g., in private channels).
//! 3. Loads localization-specific credit data from the database.
//! 4. Concatenates all credit descriptions into a single string.
//! 5. Constructs an `EmbedContent` instance with the localized title and the combined descriptions.
//! 6. Returns the embed content wrapped in a `Result`.
//!
//! ### Usage:
//! This command is expected to be triggered when a user performs a specific slash command to view credits,
//! such as `/credits`. The resulting embed provides information formatted with localization support.
//!
//! ### Dependencies:
//! The implementation depends on various modules and traits:
//! - `Command`: Trait representing commands.
//! - `CommandRun, EmbedContent, EmbedType, SlashCommand`: Support structures for building interactable content.
//! - `load_localization_credit`: Function to load localized credit data.
//!
//! ### Error Handling:
//! - Propagates errors using `anyhow::Result` in cases where localization data fails to load or any other runtime issue occurs.
use anyhow::Result;

use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::structure::message::bot::credit::load_localization_credit;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// Represents a command structure for handling credit operations in the context of a Discord bot.
///
/// # Fields
///
/// * `ctx` - An instance of `SerenityContext` that provides the context of the running bot,
///           including access to the HTTP client, cache, and other utilities necessary for
///           interacting with the Discord API.
/// * `command_interaction` - The `CommandInteraction` object representing the interaction data
///                            triggered by a user. It contains information about the command
///                            invoked, its arguments, and the user who initiated the interaction.
///
pub struct CreditCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for CreditCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A shared reference to the `SerenityContext`.
	///
	/// # Examples
	/// ```rust
	/// let context = my_instance.get_ctx();
	/// ```
	///
	/// This method allows you to access the context of the bot, which provides tools for
	/// interacting with the Discord API, such as sending messages, managing guilds, and more.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// This method provides access to the `command_interaction` field of the struct it is implemented on.
	///
	/// # Returns
	/// A shared reference to the `CommandInteraction` instance (`&CommandInteraction`).
	///
	/// # Example
	/// ```rust
	/// let command_interaction = my_struct.get_command_interaction();
	/// // Use command_interaction as needed
	/// ```
	///
	/// # Notes
	/// - Ensure the returned reference is not used after the associated struct is dropped to avoid undefined behavior.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and constructs a list of embed contents based on localized credit data.
	///
	/// This function retrieves localization data for credits specific to a guild (or a default fallback if
	/// no guild ID is present). It constructs an embed content object using the localized data, which includes
	/// a title and a concatenated description of all credits.
	///
	/// # Returns
	/// * `Result<Vec<EmbedContent<'_, '_>>>` - A vector containing a single `EmbedContent` object if successful, or 
	/// an error if something went wrong during data retrieval or processing.
	///
	/// # Steps:
	/// 1. Context (`ctx`) is retrieved using `self.get_ctx()`.
	/// 2. The `BotData` instance is fetched, including its configuration details.
	/// 3. The `guild_id` is extracted from the command interaction; defaults to `"0"` if none exists.
	/// 4. Localized credit information is fetched asynchronously by calling `load_localization_credit`, 
	/// passing the `guild_id` and database configuration.
	/// 5. Concatenate descriptions from all retrieved credits into a single string.
	/// 6. Create an `EmbedContent` instance with the retrieved title and constructed description, setting the
	/// `command_type` to `EmbedType::First`.
	/// 7. Return the constructed `EmbedContent` wrapped in a vector.
	///
	/// # Errors
	/// This function will return an error if:
	/// - The localization data cannot be fetched asynchronously.
	/// - Other unexpected issues occur while constructing the embed contents.
	///
	/// # Example Usage
	/// ```rust
	/// let contents = some_instance.get_contents().await;
	/// match contents {
	///     Ok(embeds) => {
	///         // Use the retrieved embed contents
	///     },
	///     Err(err) => {
	///         eprintln!("Error fetching embed contents: {:?}", err);
	///     }
	/// }
	/// ```
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the guild ID from the command interaction or use "0" if it does not exist
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings for the credits
		let credit_localised = load_localization_credit(guild_id, config.db.clone()).await?;

		// Construct a description by concatenating the descriptions of all credits
		let mut desc: String = "".to_string();

		for x in credit_localised.credits {
			desc += x.desc.as_str()
		}

		let embed_content = EmbedContent::new(credit_localised.title)
			.description(desc)
			.command_type(EmbedType::First);
		
		Ok(vec![embed_content])
	}
}
