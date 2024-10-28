use crate::command::command_trait::{Command, SlashCommand};
use crate::components::anilist::list_all_activity::get_formatted_activity_list;
use crate::config::Config;
use crate::constant::ACTIVITY_LIST_LIMIT;
use crate::database::activity_data::Column;
use crate::database::prelude::ActivityData;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;
use anyhow::{anyhow, Result};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateButton,
	CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use std::sync::Arc;
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
}

impl SlashCommand for ListAllActivity {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let list_activity_localised_text =
		load_localization_list_activity(guild_id, config.db.clone()).await?;

	let guild_id = command_interaction
		.guild_id
		.ok_or(anyhow!("Could not get the id of the guild"))?;

	let builder_message = Defer(CreateInteractionResponseMessage::new());

	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

	let list = ActivityData::find()
		.filter(Column::ServerId.eq(guild_id.to_string()))
		.all(&connection)
		.await?;

	let len = list.len();

	let next_page = 1;

	let activity: Vec<String> = get_formatted_activity_list(list, 0);

	let join_activity = activity.join("\n");

	let builder_message = get_default_embed(None)
		.title(list_activity_localised_text.title)
		.description(join_activity);

	let mut response = CreateInteractionResponseFollowup::new().embed(builder_message);

	trace!("{:?}", len);

	trace!("{:?}", ACTIVITY_LIST_LIMIT);

	if len > ACTIVITY_LIST_LIMIT as usize {
		response = response.button(
			CreateButton::new(format!("next_activity_{}", next_page))
				.label(&list_activity_localised_text.next),
		)
	}

	let _ = command_interaction
		.create_followup(&ctx.http, response)
		.await?;

	Ok(())
}
