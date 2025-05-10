//! Documentation for the `GivePremiumSubCommand` struct and its implementation.
//!
//! This module defines the `GivePremiumSubCommand` struct, which implements the `Command` trait. 
//! This command is used to assign a premium subscription to a specified user. It interacts with 
//! Serenity's context and Discord's HTTP API to perform the operation.
//!
//! # Structs
//!
//! ## `GivePremiumSubCommand`
//! A struct that represents the command to grant a premium subscription. It contains:
//! - `ctx`: Serenity's context object for interacting with the Discord API.
//! - `command_interaction`: Represents the command interaction details triggered by the user.
//!
//! # Methods
//!
//! ## `get_ctx`
//! Returns a reference to the Serenity context.
//!
//! ## `get_command_interaction`
//! Returns a reference to the command interaction.
//!
//! ## `get_contents`
//! Asynchronously generates and executes the premium subscription granting logic. Upon successful execution, it
//! returns a vector of `EmbedContent`, containing the operation success message.
//!
//! # Logic Flow
//!
//! - Extract user and subscription configuration options from the command interaction.
//! - Validate the provided input using predefined configurations.
//! - Fetch the list of available SKUs (Stock Keeping Units) via Discord's HTTP API and verify the subscription ID's validity.
//! - Assign the subscription to the target user by calling the `create_test_entitlement` method.
//! - Load localizations and prepare a success message embed to return as the command output.
//!
//! # Error Handling
//!
//! - This implementation uses the `anyhow` crate for error handling.
//! - Errors are returned if key information (like user or subscription options) is missing or invalid.
//! - Validation ensures that the subscription ID matches available SKUs.
//! - If any interaction with the Discord API fails, errors are propagated using `anyhow`.
//!
//! # Dependencies
//!
//! - `anyhow`: Provides error handling capabilities for the command execution.
//! - Serenity: Used to interact with the Discord API.
//! - `small_fixed_array::FixedString`: Used to handle fixed-size string operations.
//! - A data structure (`BotData`) to access configuration and localization functionality.
//!
//! # Example
//!
//! ```rust
//! use serenity::all::{Context, CommandInteraction};
//!
//! let ctx: Context = // Get Serenity context;
//! let command_interaction: CommandInteraction = // Fetch the command interaction;
//!
//! let command = GivePremiumSubCommand {
//!     ctx,
//!     command_interaction,
//! };
//!
//! let result = command.get_contents().await;
//! if let Ok(contents) = result {
//!     // Process embed contents
//! }
//! ```
//!
//! # Notes
//!
//! - The `get_contents` method assumes the presence of a bot configuration containing SKU information 
//!   and a localization database for success messages.
//! - This implementation uses Discord's test entitlement creation API for granting the subscription.
//!
//! # Fields
//!
//! - `ctx`: SerenityContext - The current Serenity context for this interaction.
//! - `command_interaction`: CommandInteraction - A representation of the user's command interaction.
use anyhow::{Result, anyhow};

use crate::command::command_trait::{Command, CommandRun, EmbedContent};
use crate::event_handler::BotData;
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use crate::structure::message::management::give_premium_sub::load_localization_give_premium_sub;
use serenity::all::{CommandInteraction, Context as SerenityContext, EntitlementOwner};
use small_fixed_array::FixedString;

/// A struct representing the `GivePremiumSubCommand`.
///
/// This struct is used to handle the logic for a command interaction
/// that grants a premium subscription to a user.
///
/// # Fields
///
/// * `ctx` - The context instance (`SerenityContext`) of the bot, 
///           which provides access to the Discord API and bot state.
/// * `command_interaction` - The command interaction instance (`CommandInteraction`)
///                           that contains the details and data of the command invoked.
pub struct GivePremiumSubCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GivePremiumSubCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) which can be used
	/// to interact with the Discord API, send messages, retrieve data, or perform
	/// other actions within the context of the bot.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // You can now use `context` to interact with the Discord API.
	/// ```
	///
	/// # Notes
	/// - The returned reference will remain valid as long as the instance (`self`)
	///   from which it was retrieved is not dropped.
	///
	/// # Panics
	/// This function does not panic.
	///
	/// # Safety
	/// This function is inherently safe as it provides an immutable reference to
	/// the context, ensuring no mutable borrowing occurs.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance.
	///
	/// # Example
	/// ```rust
	/// let interaction = object.get_command_interaction();
	/// // Use `interaction` as needed
	/// ```
	///
	/// # Notes
	/// - This method provides a read-only reference to the `CommandInteraction` field.
	/// - Ensure the lifetime of the returned reference aligns with the lifetime of the parent object.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the contents required for creating a response that indicates the 
	/// successful processing of a subscription-based entitlement for a user.
	///
	/// This function performs the following tasks:
	/// 1. Extracts the necessary context and configuration data from the bot.
	/// 2. Parses interaction data to fetch the 'user' and 'subscription' options submitted by the user.
	/// 3. Verifies if the provided subscription ID is valid against the list of available SKUs (Stock Keeping Units).
	/// 4. Creates a test entitlement for the specified subscription ID and associated user.
	/// 5. Loads localized strings for the response and generates an embed with the success message.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` containing a vector of embed contents for a successful operation.
	/// - `Err(anyhow::Error)` if there is an issue during any step, such as missing options, invalid subscription IDs, or operational failures.
	///
	/// # Errors
	/// - Returns an error if the 'user' or 'subscription' options are missing from the command interaction.
	/// - Fails if the given subscription ID is not found in the list of current SKUs.
	/// - Errors out if unable to create a test entitlement or load the localization.
	///
	/// # Examples
	/// ```rust
	/// // Example usage within an async context:
	/// let contents = get_contents().await;
	/// match contents {
	///     Ok(embed_contents) => {
	///         // Handle the embed contents, e.g., send a response
	///     }
	///     Err(err) => {
	///         // Handle the error, log failure, or inform the user
	///     }
	/// }
	/// ```
	///
	/// # Dependencies
	/// - The function relies on the following external functionality:
	///   - `get_option_map_user` and `get_option_map_string` for parsing options from the interaction.
	///   - Contextual structures like `ctx`, `BotData`, and HTTP client methods for API calls.
	///   - Utilities to generate localization strings and embed content.
	///
	/// # Note
	/// - This function assumes that the bot's configuration and HTTP client are correctly implemented and operational.
	/// - The guild ID must be available for localization purposes, and the subscription ID must correspond to an active SKU.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = &bot_data.config;

		let map = get_option_map_user(command_interaction);

		let user = *map
			.get(&FixedString::from_str_trunc("user"))
			.ok_or(anyhow!("No option for user"))?;

		let map = get_option_map_string(command_interaction);

		let subscription = map
			.get(&FixedString::from_str_trunc("subscription"))
			.ok_or(anyhow!("No option for subscription"))?
			.clone();

		let skus = ctx.http.get_skus().await?;

		let skus_id: Vec<String> = skus.iter().map(|sku| sku.id.to_string()).collect();

		if !skus_id.contains(&subscription) {
			Err(anyhow!("Invalid sub id"))?
		}

		let mut sku_id = Default::default();

		for sku in skus {
			if sku.id.to_string() == subscription {
				sku_id = sku.id;
			}
		}

		let _ = ctx
			.http
			.create_test_entitlement(sku_id, EntitlementOwner::User(user))
			.await?;

		let localization = load_localization_give_premium_sub(
			command_interaction.guild_id.unwrap().to_string(),
			config.db.clone(),
		)
			.await?;

		let embed_content = EmbedContent::new(String::default()).description(
			localization
				.success
				.replace("{user}", &user.to_string())
				.replace("{subscription}", &subscription),
		);
		
		Ok(vec![embed_content])
	}
}