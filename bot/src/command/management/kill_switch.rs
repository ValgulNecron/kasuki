//! The `KillSwitchCommand` struct implements a Discord slash command for managing
//! guild-specific module states in the application. It updates the module's "kill-switch"
//! state in the database and provides feedback to the user.
//!
//! # Fields
//! - `ctx`: The Serenity context that provides access to the bot's state and utilities.
//! - `command_interaction`: Represents the slash command interaction received from Discord.
//!
//! # Traits Implementations
//!
//! ## `Command`
//! This trait contains methods that are necessary for processing the slash command.
//!
//! ### `get_ctx`
//! Retrieves the reference to the `SerenityContext`.
//!
//! ### `get_command_interaction`
//! Retrieves the reference to the `CommandInteraction`.
//!
//! ### `get_contents`
//! Asynchronously processes the command and executes the kill-switch functionality.
//!
//! # `get_contents` Method Details
//!
//! - Retrieves configuration and context data to process the command.
//! - Determines the module name and state options from the interaction inputs.
//! - Loads guild-specific localization settings for the kill-switch description.
//! - Connects to the database and updates the state of the specified module.
//! - Composes an embed containing feedback for the user, e.g., enabling/disabling the module.
//!
//! ## Expected Errors
//! - If the `name` option is missing from the interaction.
//! - If the `state` option is missing from the interaction.
//! - If the module name provided in the interaction doesn't match any known modules.
//! - If a kill-switch row for the guild-specific ID is not found in the database.
//! - If database or localization read/write errors occur.
//!
//! ## Supported Modules
//! - `ANILIST`
//! - `AI`
//! - `GAME`
//! - `NEW_MEMBER`
//! - `ANIME`
//! - `VN`
//!
//! ## Return Value
//! - Returns a vector of `EmbedContent` instances containing localized feedback about the
//!   operation success or failure.
//!
//! ## Usage Example
//! ```rust
//! let command = KillSwitchCommand {
//!     ctx: context.clone(),
//!     command_interaction: interaction.clone(),
//! };
//!
//! if let Ok(embeds) = command.get_contents().await {
//!     // Send embeds as a response to the user.
//! }
//! ```
use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
use crate::database::kill_switch::{ActiveModel, Column};
use crate::database::prelude::KillSwitch;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::command::{get_option_map_boolean, get_option_map_string};
use crate::structure::message::management::kill_switch::load_localization_kill_switch;
use anyhow::{Result, anyhow};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, IntoActiveModel};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

/// A struct representing a kill switch command to be executed within the context of a Discord bot.
///
/// The `KillSwitchCommand` encapsulates the necessary data for handling a specific command interaction within
/// the Discord bot framework, using the Serenity library.
///
/// # Fields
/// * `ctx` - The current [`SerenityContext`] that provides access to the bot's state, data, and event handling.
/// * `command_interaction` - The [`CommandInteraction`] instance representing the trigger for this command,
///   containing details such as the command name, user input, and associated metadata.
///
/// # Usage
/// Use this struct to handle and process a kill switch command when interacting with Discord commands.
///
/// # Example
/// ```
/// let kill_switch_command = KillSwitchCommand {
///     ctx,
///     command_interaction,
/// };
/// // Process the kill switch logic here
/// ```
///
/// # Dependencies
/// This struct relies on the `Serenity` library's context and interaction modules.
///
/// [`SerenityContext`]: https://docs.rs/serenity/latest/serenity/prelude/type.Context.html
/// [`CommandInteraction`]: https://docs.rs/serenity/latest/serenity/model/interactions/application_command/struct.CommandInteraction.html
pub struct KillSwitchCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for KillSwitchCommand {
	/// Retrieves a reference to the `SerenityContext`.
	///
	/// This function returns a reference to the `SerenityContext` held by the current instance.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` associated with `self`.
	///
	/// # Examples
	///
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use the context for further operations
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance stored within the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance.
	///
	/// # Examples
	/// ```
	/// let command_interaction = my_object.get_command_interaction();
	/// // Use `command_interaction` as needed
	/// ```
	///
	/// This method is useful for accessing the `CommandInteraction` object
	/// encapsulated within the struct without taking ownership.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the contents based on the provided user interaction and settings.
	///
	/// This function performs the following tasks:
	/// 1. Fetches the current application context (`ctx`) and bot-wide data (`BotData`).
	/// 2. Retrieves the guild ID from the command interaction or defaults to "0" if unavailable.
	/// 3. Extracts the `name` and `state` command options from the command's inputs.
	/// 4. Loads localization settings based on the guild ID.
	/// 5. Connects to the database to retrieve and update the `KillSwitch` row corresponding to the module name.
	/// 6. Toggles the specified module's state (`enabled` or `disabled`) and persists the changes in the database.
	/// 7. Prepares and returns an embedded message (`EmbedContent`) summarizing the action performed.
	///
	/// # Parameters
	/// - `&self`: A reference to the instance of the calling object.
	///
	/// # Returns
	/// - `Result<Vec<EmbedContent<'_, '_>>>`: A result containing a vector of `EmbedContent`, which is used to display success messages.
	///   - If successful, returns a single message explaining the state update for a specific module.
	///   - If an error occurs, returns an error detailing the failure.
	///
	/// # Errors
	/// This function returns an error in any of the following cases:
	/// - `name` or `state` option is missing from the provided command interaction.
	/// - Database connection fails.
	/// - The specified module name does not match any recognized modules.
	/// - The `KillSwitch` for the given guild cannot be found.
	/// - Database update operations fail while saving the change.
	///
	/// # Modules
	/// This function supports the following module names:
	/// - `"ANILIST"`
	/// - `"AI"`
	/// - `"GAME"`
	/// - `"NEW_MEMBER"`
	/// - `"ANIME"`
	/// - `"VN"`
	///
	/// If an unsupported module name is provided, an error is returned.
	///
	/// # Example Output
	/// - For enabling the Anime module:
	///     - Returns an `EmbedContent` with the module state set to "on".
	/// - For disabling the AI module:
	///     - Returns an `EmbedContent` with the module state set to "off".
	///
	/// # Dependencies
	/// - Relies on `sea_orm` for database operations.
	/// - Utilizes localization data based on the guild ID to provide appropriate messages.
	/// - Requires interaction context to extract user commands and options.
	///
	/// # Notes
	/// Ensure that the database setup is correct and the necessary tables (`KillSwitch`) exist with the expected schema.
	/// The localization function (`load_localization_kill_switch`) must support fallback behaviors for missing translations.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let map = get_option_map_string(command_interaction);

		let module = map
			.get(&FixedString::from_str_trunc("name"))
			.ok_or(anyhow!("No option for name"))?;

		let module_localised =
			load_localization_kill_switch(guild_id.clone(), config.db.clone()).await?;

		let map = get_option_map_boolean(command_interaction);

		let state = *map
			.get(&FixedString::from_str_trunc("state"))
			.ok_or(anyhow!("No option for state"))?;

		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

		let mut row = KillSwitch::find()
			.filter(Column::GuildId.eq("0"))
			.one(&connection)
			.await?
			.ok_or(anyhow!("KillSwitch not found"))?;

		match module.as_str() {
			"ANILIST" => row.anilist_module = state,
			"AI" => row.ai_module = state,
			"GAME" => row.game_module = state,
			"NEW_MEMBER" => row.new_members_module = state,
			"ANIME" => row.anime_module = state,
			"VN" => row.vn_module = state,
			_ => {
				return Err(anyhow!("The module specified does not exist"));
			},
		}

		let active_model: ActiveModel = row.into_active_model();

		active_model.update(&connection).await?;

		let desc = if state {
			module_localised.on
		} else {
			module_localised.off
		};

		let embed_content = EmbedContent::new(module.clone())
			.description(desc)
			.command_type(EmbedType::First);

		Ok(vec![embed_content])
	}
}
