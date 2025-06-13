//! This module defines and implements the `LeaveCommand` structure, which is a concrete
//! implementation of the `Command` trait. The purpose of the `LeaveCommand` is to handle
//! the "leave" command functionality, primarily for managing the bot's disconnection
//! from voice channels in a guild context.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::structure::message::music::leave::load_localization_leave;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing a command used to handle a "leave" interaction in a Discord bot.
///
/// The `LeaveCommand` struct contains context and interaction data necessary for executing the command
/// when invoked by a user.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext` provides the bot's context, including connection details,
///           interaction with Discord API, and shared state.
///
/// * `command_interaction` - The `CommandInteraction` holds the details of the user's invoked command.
///                           This includes metadata (e.g., command arguments, associated user, etc.).
///
/// # Purpose
///
/// This struct is specifically designed to encapsulate the components required for handling
/// "leave" command logic, which is typically used to make the bot leave a channel, server, or
/// audio session, depending on the bot functionality.
///
/// # Example Usage
///
/// This struct would typically be initialized and passed to a handling function that executes
/// the "leave" logic:
///
/// ```rust
/// let leave_command = LeaveCommand {
///     ctx,
///     command_interaction,
/// };
/// leave_command.execute().await;
/// ```
pub struct LeaveCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LeaveCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) stored within the instance.
	/// This can be used to interact with the Discord API, send messages, retrieve guild information, etc.
	///
	/// # Examples
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use ctx to interact with the Discord API
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the associated `CommandInteraction` instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` stored within the current instance.
	///
	/// # Examples
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use `interaction` as needed...
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes content for the command interaction.
	///
	/// This function performs the following operations:
	/// 1. Retrieves the interaction context and relevant state from the `BotData`.
	/// 2. Defers the interaction response.
	/// 3. Extracts the guild ID from the command interaction, defaulting to "0" if unavailable.
	/// 4. Loads localized content for the "leave" command using the provided guild ID.
	/// 5. Validates the Lavalink client configuration and removes the player associated with the guild.
	/// 6. Removes the guild from the internal bot manager if it exists.
	/// 7. Creates an `EmbedContent` object to indicate the success of the operation and returns it.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` on success with the localized embed content.
	/// - `Err(anyhow::Error)` if any operation (such as retrieving the Lavalink client, deleting the player, or
	///   an absence of guild ID) fails.
	///
	/// # Errors
	/// - Returns an error if the Lavalink client is not configured or is disabled.
	/// - Returns an error if the guild ID cannot be retrieved from the interaction.
	/// - Propagates errors from `defer`, `load_localization_leave`, `delete_player`, or `manager.remove` operations.
	///
	/// # Async Behavior
	/// - The function contains multiple asynchronous operations, such as deferring the interaction, fetching
	///   localized content, and making calls to Lavalink or the internal manager. These operations are awaited.
	///
	/// # Example
	/// ```ignore
	/// let contents = interaction.get_contents().await?;
	/// for content in contents {
	///     println!("Embed Title: {}", content.title());
	/// }
	/// ```
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let leave_localised =
			load_localization_leave(guild_id_str, bot_data.config.db.clone()).await?;

		let manager = bot_data.manager.clone();
		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		if lava_client.is_none() {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		}
		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

		let lava_client = lava_client.unwrap();

		lava_client
			.delete_player(lavalink_rs::model::GuildId::from(guild_id.get()))
			.await?;

		if manager.get(guild_id).is_some() {
			manager.remove(guild_id).await?;
		}

		let embed_content =
			EmbedContent::new(leave_localised.title).description(leave_localised.success);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}
