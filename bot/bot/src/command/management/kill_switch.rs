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
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::command::{get_option_map_boolean, get_option_map_string};
use crate::impl_command;
use anyhow::anyhow;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, IntoActiveModel};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::kill_switch::{ActiveModel, Column};
use shared::database::prelude::KillSwitch;
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
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
#[derive(Clone)]
pub struct KillSwitchCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for KillSwitchCommand,
	get_contents = |self_: KillSwitchCommand| async move {
		let ctx = self_.get_ctx();
		let command_interaction = self_.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();

		let config = bot_data.config.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let map = get_option_map_string(command_interaction);

		let module = map
			.get(&FixedString::from_str_trunc("name"))
			.ok_or(anyhow!("No option for name"))?;
		let db_connection = bot_data.db_connection.clone();

		let lang_id = get_language_identifier(guild_id.clone(), db_connection).await;

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
			"LEVEL" => row.level_module = state,
			"MINIGAME" => row.mini_game_module = state,
			"ANIME" => row.anime_module = state,
			"VN" => row.vn_module = state,
			_ => {
				return Err(anyhow!("The module specified does not exist"));
			},
		}

		let active_model: ActiveModel = row.into_active_model();

		active_model.update(&connection).await?;

		let desc = if state {
			USABLE_LOCALES.lookup(&lang_id, "management_kill_switch-on")
		} else {
			USABLE_LOCALES.lookup(&lang_id, "management_kill_switch-off")
		};

		let embed_content = EmbedContent::new(module.clone()).description(desc);

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
);
