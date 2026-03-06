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
use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::components::anilist::list_all_activity::get_formatted_activity_list;
use crate::constant::ACTIVITY_LIST_LIMIT;
use crate::event_handler::BotData;
use anyhow::anyhow;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::activity_data::Column;
use shared::database::prelude::ActivityData;
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

#[slash_command(
	name = "list_activity", desc = "Get the list of registered activity.", command_type = ChatInput,
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn list_all_activity_command(self_: ListAllActivity) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx().clone();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction().clone();
	let _config = bot_data.config.clone();


	let guild_id = command_interaction
		.guild_id
		.ok_or(anyhow!("Could not get the id of the guild"))?;

	let connection = bot_data.db_connection.clone();
	let list = ActivityData::find()
		.filter(Column::ServerId.eq(guild_id.to_string()))
		.all(&*connection)
		.await?;
	let len = list.len();

	let activity: Vec<String> = get_formatted_activity_list(list, 0);
	let join_activity = activity.join("\n");
	let db_connection = bot_data.db_connection.clone();

	let lang = get_guild_language(guild_id.to_string(), db_connection).await;
	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
	let title = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_all_activity-title");

	let embed_content = EmbedContent::new(title).description(join_activity);
	let action_row;
	if len > ACTIVITY_LIST_LIMIT as usize {
		action_row = None
	} else {
		action_row = None
	}

	let mut embed_contents = EmbedsContents::new(vec![embed_content]);
	if let Some(action_row) = action_row {
		embed_contents = embed_contents.action_row(action_row);
	}

	Ok(embed_contents)
}
