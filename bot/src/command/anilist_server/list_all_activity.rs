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
use crate::impl_command;
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;
use anyhow::anyhow;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};

#[derive(Clone)]
pub struct ListAllActivity {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for ListAllActivity,
	get_contents = |self_: ListAllActivity| async move {
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();
		let _config = bot_data.config.clone();

		self_.defer().await?;

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
		let db_connection = bot_data.db_connection.clone();

		let list_activity_localised_text =
			load_localization_list_activity(guild_id.to_string(), db_connection).await?;

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
);
