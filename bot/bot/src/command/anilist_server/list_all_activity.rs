use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::components::anilist::list_all_activity::get_formatted_activity_list;
use crate::constant::ACTIVITY_LIST_LIMIT;
use anyhow::anyhow;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::activity_data::Column;
use shared::database::prelude::ActivityData;
use shared::localization::USABLE_LOCALES;

#[slash_command(
	name = "list_activity", desc = "Get the list of registered activity.", command_type = ChatInput,
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn list_all_activity_command(self_: ListAllActivity) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("Could not get the id of the guild"))?;

	let list = ActivityData::find()
		.filter(Column::ServerId.eq(guild_id.to_string()))
		.all(&*cx.db)
		.await?;
	let len = list.len();

	let activity: Vec<String> = get_formatted_activity_list(list, 0);
	let join_activity = activity.join("\n");

	let lang_id = cx.lang_id().await;
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
