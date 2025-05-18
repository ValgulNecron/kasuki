//! `ListAllActivity` is a struct that implements the `Command` trait. It is responsible for fetching and displaying
//! a list of activities for a specific server (guild) in the Discord context.
//!
//! Fields:
//! - `ctx` - The Serenity context object representing the current Discord context.
//! - `command_interaction` - The interaction object representing the command interaction from a user.
//!
//! Implements:
//! - `Command`: Provides methods for accessing context, interaction details, and generating embed content.
//!
//! Methods:
//!
//! - `get_ctx`:
//!     - Returns a reference to the `SerenityContext` associated with this command.
//!     - This allows access to the shared bot context, which includes data like configuration and database connections.
//!
//! - `get_command_interaction`:
//!     - Returns a reference to the `CommandInteraction` associated with this command.
//!     - Useful for accessing details about the command interaction (e.g., guild ID, user ID, etc.).
//!
//! - `get_contents`:
//!     - Fetches the list of activities from the database for the guild associated with the command interaction.
//!     - Provides the formatted list as embed content that can be displayed as a follow-up message on Discord.
//!     - If the activity list exceeds the set limit (`ACTIVITY_LIST_LIMIT`), it will include a "Next" button to paginate the results.
//!
//!       Workflow:
//!       1. Retrieve the guild ID from the `command_interaction`.
//!       2. Fetch associated activities from the database using the `ActivityData` model.
//!       3. Format the activity list using the `get_formatted_activity_list` function, converting database entries into a readable format.
//!       4. Retrieve localized text for displaying the title and next page label.
//!       5. Construct the embed content with the formatted activity list and attach a pagination button if necessary.
//!
//!       Returns:
//!       - `Result<Vec<EmbedContent<'_, '_>>>`: A vector of `EmbedContent` containing the activity list for display.
//!       - Errors if the guild ID cannot be retrieved or if database interaction fails.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{
	ButtonV1, CommandType, ComponentVersion, ComponentVersion1, EmbedContent, EmbedsContents,
};
use crate::components::anilist::list_all_activity::get_formatted_activity_list;
use crate::constant::ACTIVITY_LIST_LIMIT;
use crate::database::activity_data::Column;
use crate::database::prelude::ActivityData;
use crate::event_handler::BotData;
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;
use anyhow::{Result, anyhow};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateActionRow, CreateButton,
};
use std::borrow::Cow;

/// Represents a structure to list all activity within a Discord bot command context.
///
/// This struct is used to handle interactions received from the user via Discord commands.
/// It contains the context of the bot and the specific command interaction details.
///
/// # Fields
///
/// * `ctx` - The bot's runtime context, provided by Serenity. It allows access to bot-related operations
///   such as retrieving data, sending messages, managing the bot state, or performing API calls.
/// * `command_interaction` - Represents the command interaction triggered by a user. It contains details
///   about the interaction, such as the command name, arguments, options, and the user who triggered the command.
///
/// # Example
///
/// ```rust
/// use serenity::prelude::*;
/// use serenity::model::prelude::*;
///
/// let activity = ListAllActivity { ctx: serenity_context, command_interaction: command_interaction };
/// ```
///
/// This struct is typically used in the command execution lifecycle to process and respond to user commands.
pub struct ListAllActivity {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ListAllActivity {
	/// Retrieves a reference to the `SerenityContext` associated with this instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) that is stored
	/// within the current instance.
	///
	/// # Example
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use `ctx` for further operations.
	/// ```
	///
	/// This method is useful for accessing the Discord bot's context,
	/// which can be used to interact with the API and handle bot operations.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` field of the instance.
	///
	/// # Examples
	/// ```rust
	/// // Assuming `instance` is an instance of a struct that has this method
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes a list of activity data from the database
	/// for display purposes, returning a list of embed contents.
	///
	/// # Returns
	/// * `Result<Vec<EmbedContent<'_, '_>>>` - A vector containing `EmbedContent` objects which
	///   represent the formatted activities.
	///
	/// # Workflow
	/// 1. Retrieve the execution context and the command interaction information.
	/// 2. Attempt to determine the guild ID from the interaction; return an error if not found.
	/// 3. Access the bot data and duplicate the database connection.
	/// 4. Query the database for activities that are associated with the guild ID.
	/// 5. Format the fetched activities into a localized, human-readable list.
	///
	/// # Parameters
	/// * `&self` - A reference to the associated struct calling this method.
	///
	/// # Intermediate Computations
	/// - Fetches activity entries from the database using the guild ID.
	/// - Checks if the activity list exceeds a predefined limit and, if so, prepares
	///   action buttons for pagination.
	///
	/// # Returns
	/// - A localized embed for each activity list, including its title, description, and optional buttons.
	///
	/// # Error Handling
	/// - Returns an error if:
	///   * The guild ID cannot be fetched from the interaction.
	///   * The database query fails.
	///   * Any localization or formatting errors occur.
	///
	/// # Examples
	/// ```rust
	/// let result = my_struct.get_contents().await;
	/// match result {
	///     Ok(embed_contents) => {
	///         // Do something with the embed contents
	///     },
	///     Err(e) => {
	///         eprintln!("Failed to fetch activity contents: {}", e);
	///     }
	/// }
	/// ```
	///
	/// # Note
	/// This function is asynchronous and should be awaited in an appropriate async runtime environment.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();

		self.defer().await?;

		let guild_id = command_interaction
			.guild_id
			.ok_or(anyhow!("Could not get the id of the guild"))?;

		let connection = bot_data.db_connection.clone();
		let list = ActivityData::find()
			.filter(Column::ServerId.eq(guild_id.to_string()))
			.all(&*connection)
			.await?;
		let len = list.len();
		let next_page = 1;

		let activity: Vec<String> = get_formatted_activity_list(list, 0);
		let join_activity = activity.join("\n");

		let list_activity_localised_text =
			load_localization_list_activity(guild_id.to_string(), config.db.clone()).await?;

		let embed_content =
			EmbedContent::new(list_activity_localised_text.title).description(join_activity);
		let action_row;
		if len > ACTIVITY_LIST_LIMIT as usize {
			let buttons = vec![
				ButtonV1::new(list_activity_localised_text.next)
					.custom_id(format!("next_activity_{}", next_page)),
			];
			let v1 = ComponentVersion1::buttons(buttons);
			action_row = Some(ComponentVersion::V1(v1));
		} else {
			action_row = None
		}

		let mut embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
		if let Some(action_row) = action_row {
			embed_contents = embed_contents.action_row(action_row);
		}

		Ok(embed_contents)
	}
}
