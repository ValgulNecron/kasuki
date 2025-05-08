use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType, SlashCommand};
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
use tracing::trace;

pub struct ListAllActivity {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ListAllActivity {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
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
		
		let mut embed_content = EmbedContent::new(list_activity_localised_text.title)
			.description(join_activity)
			.command_type(EmbedType::Followup);
		if len > ACTIVITY_LIST_LIMIT as usize {
			embed_content = embed_content.action_row(vec![CreateActionRow::Buttons(Cow::from(vec![
				CreateButton::new(format!("next_activity_{}", next_page))
					.label(list_activity_localised_text.next),
			]))]);
		}
		
		Ok(vec![embed_content])
	}
}
