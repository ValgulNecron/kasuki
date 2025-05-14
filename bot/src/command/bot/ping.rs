//! The `PingCommand` struct represents a command implementation for handling
//! a "ping" command within a Discord bot context. It integrates with Serenity
//! to interact with Discord's API.
//!
//! Fields:
//! - `ctx`: The `SerenityContext` for accessing the bot's context and runtime data.
//! - `command_interaction`: The `CommandInteraction` received from a user input.
//!
//! This struct implements the `Command` trait, which provides methods for retrieving
//! execution context, processing the command interaction, and constructing the
//! response as embedded content.
use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::structure::message::bot::ping::load_localization_ping;
use anyhow::{Result, anyhow};
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing a PingCommand in the bot's command handling system.
///
/// The `PingCommand` contains all the necessary context and interaction data
/// required to process and respond to a ping command issued by a user.
///
/// # Fields
///
/// * `ctx` - A `SerenityContext` instance, which provides access to the bot's
///   runtime state, such as data, configuration, and shard management. This is
///   used to interact with the Discord API as needed while handling the command.
///
/// * `command_interaction` - A `CommandInteraction` instance, containing
///   information about the specific command interaction triggered by the user,
///   including arguments, user details, and the command ID.
///
/// # Purpose
///
/// This struct encapsulates the required data for handling a "ping" command,
/// which is typically used to measure the bot's responsiveness or provide a
/// simple acknowledgment response.
pub struct PingCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for PingCommand {
	/// Retrieves a reference to the `SerenityContext` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`), which can be used
	/// to interact with the Discord API or handle various bot operations.
	///
	/// # Example
	/// ```rust
	/// let context = my_object.get_ctx();
	/// // Use the context for Discord API interactions
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	///
	/// This method provides read-only access to the `CommandInteraction`
	/// stored in the instance.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves a vector containing `EmbedContent` based on the status and latency of a Discord bot shard.
	///
	/// # Returns
	/// - A `Result` containing a vector of `EmbedContent` on success or an `anyhow::Error` on failure.
	///
	/// # Errors
	/// This function will return an error in the following cases:
	/// - The shard manager could not be retrieved from the bot's context.
	/// - The shard information could not be found for the current shard ID.
	/// - Localization strings for the ping command could not be loaded.
	///
	/// # Description
	/// 1. Retrieves necessary data from the bot's context, such as the bot configuration and shard manager.
	/// 2. Fetches the current guild ID from the command's interaction context. If no guild ID is found, a default value of "0" is used.
	/// 3. Loads localized strings associated with the `ping` command, specific to the guild ID and database configuration.
	/// 4. Fetches the shard manager, ensuring it is accessible and valid. If the shard manager is unavailable, an error is returned.
	/// 5. Extracts shard-specific information such as shard ID, latency (formatted as milliseconds), and connection stage.
	///    - Formats the latency appropriately if available, otherwise defaults to `"?,??ms"`.
	/// 6. Generates an embed description by replacing placeholders (`$shard$`, `$latency$`, `$status$`) with dynamic values.
	/// 7. Constructs an `EmbedContent` object with the localized title and formatted description.
	/// 8. Returns a vector containing the constructed `EmbedContent`.
	///
	/// # Example
	/// ```rust
	/// let embed_contents = my_context.get_contents().await?;
	/// for embed in embed_contents {
	///     println!("Embed Title: {}", embed.title());
	///     println!("Embed Description: {}", embed.description());
	/// }
	/// ```
	///
	/// # Dependencies
	/// - `BotData`: Accesses bot-specific state, including configuration and shard manager.
	/// - `EmbedContent`: Represents data specified for Discord embed generation.
	/// - `EmbedType`: Enumerates the type of embed (in this case, `EmbedType::First`).
	/// - A localization utility (`load_localization_ping`) for loading localized strings.
	///
	/// # Notes
	/// - Ensure that the `BotData` structure and `load_localization_ping` function are correctly implemented to support your bot's database and configuration design.
	/// - `ctx.shard_id` is used to associate the command interaction with the appropriate shard.
	///
	/// # See Also
	/// - `ShardManager` for managing bot shards.
	/// - `EmbedContent` for creating embeddable content for Discord messages.
	/// - `anyhow` crate for error handling.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = &bot_data.config;

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized ping strings
		let ping_localised = load_localization_ping(guild_id, config.db.clone()).await?;

		let guard = ctx.data::<BotData>().shard_manager.clone();
		let guard = guard.read().await;
		let manager = guard.clone();
		drop(guard);
		let shard_manager = match manager {
			Some(shard_manager) => shard_manager.clone(),
			None => {
				return Err(anyhow!("failed to get the shard manager"));
			},
		};

		// Retrieve the shard ID from the context
		let shard_id = ctx.shard_id;
		// Retrieve the shard runner info from the shard manager
		let (latency, stage) = {
			let shard_runner_info = shard_manager
				.get(&shard_id)
				.ok_or(anyhow!("failed to get the shard info"))?;
			// Format the latency as a string
			let (info, _) = shard_runner_info.value();
			let latency = match info.latency {
				Some(latency) => format!("{:.2}ms", latency.as_millis()),
				None => "?,??ms".to_string(),
			};

			// Retrieve the stage of the shard runner
			let stage = info.stage.to_string();
			drop(shard_runner_info);
			(latency, stage)
		};

		let embed_content = EmbedContent::new(ping_localised.title)
			.description(
				ping_localised
					.desc
					.replace("$shard$", shard_id.to_string().as_str())
					.replace("$latency$", latency.as_str())
					.replace("$status$", &stage),
			)
			.command_type(EmbedType::First);

		Ok(vec![embed_content])
	}
}
