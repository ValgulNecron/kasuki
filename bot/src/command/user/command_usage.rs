//! The `CommandUsageCommand` struct represents a specific implementation of a bot command in the
//! Discord bot framework using the `serenity` library. This command is used to display the usage
//! statistics of commands for a specific user.
//!
//! The struct contains the following fields:
//! - `ctx`: A `SerenityContext` instance that represents the context in which the bot operates.
//! - `command_interaction`: A `CommandInteraction` instance that represents the interaction data
//!   received from the user.
//!
//! Implements the `Command` trait which defines behavior for executing or interacting with a bot command.
//!
//! Example usage:
//! ```
//! let command_usage = CommandUsageCommand {
//!     ctx: serenity_context,
//!     command_interaction: command_interaction_data
//! };
//! let embed_contents = command_usage.get_contents().await?;
//! ```
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::user::avatar::get_user_command;
use crate::event_handler::{BotData, RootUsage};
use crate::structure::message::user::command_usage::load_localization_command_usage;
use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tokio::sync::RwLockReadGuard;

/// A struct representing a command usage event within a Discord application.
///
/// This struct contains essential context and interaction details related to
/// a specific command executed within the Discord bot framework, Serenity.
///
/// # Fields
/// - `ctx`: Contains the context of the command. This includes the state of
///   the bot, such as data, cache, and shard information, which can be used
///   for further interaction with the Discord API.
/// - `command_interaction`: Represents the interaction received from Discord
///   for the specific command. This includes data about the user input,
///   the invoked command, and any associated options or parameters.
///
/// # Usage
/// This struct is ideal for handling, processing, and responding to command
/// interactions with sufficient context and command details.
///
/// # Example
/// ```rust
/// use your_crate::CommandUsageCommand;
///
/// let command_usage = CommandUsageCommand {
///     ctx: some_context,
///     command_interaction: some_interaction,
/// };
///
/// // Use the fields `ctx` and `command_interaction` to process the command.
/// ```
pub struct CommandUsageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for CommandUsageCommand {
	/// Retrieves a reference to the `SerenityContext` object.
	///
	/// This function provides access to the `SerenityContext` associated with the current instance.
	/// The `SerenityContext` allows interaction with the Discord API and context-dependent operations.
	///
	/// # Returns
	/// A reference to the `SerenityContext` contained within the implementing structure.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` for Discord API interactions
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` contained within the struct.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	///
	/// This method is useful when you need to access the details or properties of the
	/// `CommandInteraction` stored in the struct.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves the contents of embedded messages for displaying user-specific command usage statistics.
	///
	/// # Returns
	/// This asynchronous function returns a `Result` containing `EmbedsContents`. On success, it provides the constructed embed(s)
	/// that can be displayed as part of bot interactions. On failure, it returns an error.
	///
	/// # How it works
	/// 1. **Fetch the User**:
	///     - Retrieves the user information (`user`) who invoked the command via `get_user_command`.
	///
	/// 2. **Bot Context and Configuration**:
	///     - Accesses necessary bot data (such as database configuration and command usage statistics)
	///       from the context object.
	///     - Retrieves the command interaction information.
	///
	/// 3. **Retrieve and Process Usage Statistics**:
	///     - Identifies the invoking user's ID and username.
	///     - Fetches the user's command usage statistics from the shared, thread-safe `command_usage` storage.
	///     - Processes usage data, tailored based on a localization system configured for the guild (server)
	///       that the command was used in.
	///
	/// 4. **Build Embeds**:
	///     - Constructs the embedded message contents based on the localized response templates.
	///     - If the user has no usage statistics, an "empty usage" message is generated.
	///     - If the user has usage statistics, iterates over the usage records and appends them to the embed's description.
	///     - Manages character limits for embeds by ensuring no embed description exceeds the allowed limit (4096 characters),
	///       splitting data across multiple embeds if necessary.
	///
	/// 5. **Return Result**:
	///     - Creates an instance of `EmbedsContents` object with the generated embed(s).
	///     - Wraps the result and returns it.
	///
	/// # Errors
	/// This function can return an error in the following cases:
	/// - If there is an issue fetching the invoking user's information.
	/// - If there is an issue loading localization data for command usage.
	///
	/// # Dependencies
	/// - `get_user_command`: Fetches the user object from the command interaction context.
	/// - `BotData`: Accesses bot-wide data, such as command statistics and database configurations.
	/// - `get_usage_for_id`: Extracts usage data for a specific user ID from the command usage records.
	/// - `load_localization_command_usage`: Loads localized templates for command statistics based on the guild (server) ID.
	///
	/// # Arguments
	/// - `&self`: The instance holding the current context (`ctx`), command interaction, and required data.
	///
	/// # Example
	/// ```rust
	/// let embed_contents = self.get_contents().await?;
	/// // Use the returned embed_contents to send a followup message to the user.
	/// ```
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let user = get_user_command(&self.ctx, &self.command_interaction).await?;
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let command_usage = bot_data.number_of_command_use_per_command.clone();
		let db_config = bot_data.config.db.clone();

		let user_id = user.id.to_string();

		let username = user.name.clone();

		let read_command_usage = command_usage.read().await;

		let usage = get_usage_for_id(&user_id, read_command_usage);

		let guild_id = command_interaction
			.guild_id
			.map(|id| id.to_string())
			.unwrap_or("0".to_string());

		let localized_command_usage = load_localization_command_usage(guild_id, db_config).await?;

		let mut embed_contents = vec![];

		let embed_content =
			EmbedContent::new(localized_command_usage.title.replace("$user$", &username));

		if usage.is_empty() {
			let inner_embed = embed_content.description(
				localized_command_usage
					.no_usage
					.replace("$user$", &username),
			);
			embed_contents.push(inner_embed);
		} else {
			let mut description = String::new();

			let mut inner_embed = embed_content.clone();

			for (command, usage_count) in &usage {
				description.push_str(
					localized_command_usage
						.command_usage
						.replace("$command$", command)
						.replace("$usage$", &usage_count.to_string())
						.as_str(),
				);

				description.push('\n');

				if description.len() > 4096 {
					let desc = description.clone();
					embed_contents.push(inner_embed.clone().description(desc));

					description.clear();

					inner_embed = embed_content.clone();
				}
			}

			if !description.is_empty() {
				embed_contents.push(inner_embed.clone().description(description));
			}
		}

		let embed_contents = EmbedsContents::new(CommandType::Followup, embed_contents);

		Ok(embed_contents)
	}
}

/// Retrieves usage statistics for a specific user ID from the given `RootUsage` data structure.
///
/// # Arguments
///
/// * `target_id` - A `&str` representing the target user ID whose usage information needs to be retrieved.
/// * `root_usage` - A `RwLockReadGuard<RootUsage>` providing read access to the `RootUsage` instance
///   which stores the overall command usage data for all users.
///
/// # Returns
///
/// A `Vec<(String, u128)>` where each element is a tuple containing:
/// - `String`: The name of the command related to the usage.
/// - `u128`: The usage value corresponding to the command.
///
/// # Workflow
///
/// 1. The function iterates through all the commands in the `command_list` of `RootUsage`.
/// 2. For each command, it examines the usage details of all users.
/// 3. If the `target_id` matches a user ID, it collects the command name and associated usage data as a tuple.
/// 4. The collected tuples are stored in a `Vec` and returned.
///
/// # Example
///
/// ```rust
/// let target_id = "user123";
/// let root_usage = obtain_root_usage(); // Assume `obtain_root_usage` returns a valid RwLockReadGuard<RootUsage>.
/// let user_usage = get_usage_for_id(target_id, root_usage);
///
/// for (command, usage) in user_usage {
///     println!("Command: {}, Usage: {}", command, usage);
/// }
/// ```
///
/// # Notes
///
/// - This function assumes that `root_usage` provides a thread-safe read lock to the underlying data structure.
/// - The result vector will be empty if `target_id` is not found in the usage data.
/// - Since the function iterates through all commands and their associated user data, it may have performance implications for large datasets.
///
/// # Dependencies
///
/// - `RwLockReadGuard` must be properly instantiated before passing it to this function.
/// - The `RootUsage` structure must include `command_list` data, where each entry associates a command with user-specific usage details.
///
/// # Safety
///
/// No mutable operations are performed; this function strictly reads from the provided data structure.
fn get_usage_for_id(
	target_id: &str, root_usage: RwLockReadGuard<RootUsage>,
) -> Vec<(String, u128)> {
	let mut usage = Vec::new();

	for (command, user_info) in root_usage.command_list.iter() {
		for (id, user_usage) in user_info.user_info.iter() {
			if id == target_id {
				usage.push((command.clone(), user_usage.usage));
			}
		}
	}

	usage
}
