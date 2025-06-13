//! The `RemoveTestSubCommand` struct represents a command for removing a user's test subscription.
//!
//! This struct implements the `Command` trait, allowing it to integrate with the bot's command system.
//!
//! # Fields
//! - `ctx`
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_user;
use crate::structure::message::management::remove_test_sub::load_localization_remove_test_sub;
use anyhow::{Result, anyhow};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseMessage,
};
use small_fixed_array::FixedString;
use tracing::error;

/// The `RemoveTestSubCommand` struct defines a structure for a specific subcommand
/// in handling interactions within a Discord bot using the Serenity library.
///
/// # Fields
///
/// * `ctx` - Represents the context of the bot, provided by the Serenity library.
///   This context contains information about the bot's state, shard, cache, and other
///   utilities required to interact with Discord's API.
///
/// * `command_interaction` - Represents the interaction data for the specific command
///   triggered by a user. It provides details such as user input, interaction ID,
///   and any parameters passed in the command.
///
/// This structure can be used for implementing logic specific to handling a "remove test"
/// subcommand within the bot's interaction system.
pub struct RemoveTestSubCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RemoveTestSubCommand {
	/// Retrieves a reference to the `SerenityContext` associated with this instance.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` (`&SerenityContext`), which provides
	/// the context needed for interacting with Discord through the Serenity library.
	///
	/// # Examples
	///
	/// ```rust
	/// let context = my_instance.get_ctx();
	/// // Use `context` to interact with Discord API.
	/// ```
	///
	/// # Notes
	/// This function borrows the `SerenityContext` immutably, ensuring the context
	/// is accessible without allowing modifications.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with this object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance.
	///
	/// # Examples
	/// ```
	/// let interaction = my_object.get_command_interaction();
	/// ```
	///
	/// This method provides access to the underlying `CommandInteraction` field
	/// of the object, allowing for read-only operations on the data.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes the contents related to user entitlements.
	///
	/// This function performs the following
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let map = get_option_map_user(command_interaction);

		let user = map.get(&FixedString::from_str_trunc("user"));

		let user = match user {
			Some(user) => user,
			None => {
				return Err(anyhow!("No user provided"));
			},
		};

		let entitlements = ctx
			.http
			.get_entitlements(Some(*user), None, None, None, None, None, None)
			.await?;

		let localization = load_localization_remove_test_sub(
			command_interaction.guild_id.unwrap().to_string(),
			config.db.clone(),
		)
		.await?;

		// defer the response
		let builder_message = Defer(CreateInteractionResponseMessage::new());

		command_interaction
			.create_response(&ctx.http, builder_message)
			.await?;

		for entitlement in entitlements {
			if let Err(e) = ctx.http.delete_test_entitlement(entitlement.id).await {
				error!("Error while deleting entitlement: {}", e);
			}
		}

		let embed_content = EmbedContent::new(String::new())
			.description(localization.success.replace("{user}", &user.to_string()));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}
