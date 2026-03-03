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
use anyhow::anyhow;

use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext, EntitlementOwner};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use small_fixed_array::FixedString;
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "give_premium_sub", desc = "Give a premium subscription to a user.",
	command_type = GuildChatInput { guild_id = 1117152661620408531 },
	permissions = [Administrator],
	args = [
		(name = "user", desc = "The user to give the subscription to.", arg_type = User, required = true, autocomplete = false),
		(name = "subscription", desc = "The subscription to give.", arg_type = String, required = true, autocomplete = true)
	],
)]
async fn give_premium_sub_command(self_: GivePremiumSubCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();

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
	let db_connection = bot_data.db_connection.clone();

	let guild_id = command_interaction.guild_id.unwrap().to_string();
	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	args.insert(Cow::Borrowed("user"), FluentValue::from(user.to_string()));
	args.insert(
		Cow::Borrowed("subscription"),
		FluentValue::from(subscription.clone()),
	);

	let success_msg =
		USABLE_LOCALES.lookup_with_args(&lang_id, "management_give_premium_sub-success", &args);

	let embed_content = EmbedContent::new(String::default()).description(success_msg);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
