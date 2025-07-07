use anyhow::anyhow;
use sea_orm::EntityTrait;use sea_orm::QueryFilter;
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::management::give_premium_sub::GivePremiumSubCommand;
use crate::database::message;
use crate::database::prelude::{Message as DatabaseMessage, Vocal as DatabaseVocal};
use crate::event_handler::BotData;
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use crate::impl_command;
use crate::structure::message::management::give_premium_sub::load_localization_give_premium_sub;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, Condition};
use serenity::all::{ChannelId, CommandInteraction, Context as SerenityContext, EntitlementOwner};
use small_fixed_array::FixedString;
#[derive(Clone)]
pub struct LevelsStatsCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for LevelsStatsCommand,
	get_contents = |self_: LevelsStatsCommand| async move {
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let db_connection = bot_data.db_connection.clone();

		let channels_id = command_interaction.guild_id.unwrap().channels(&ctx.http)
			.await?;
		let vec_channel_id: Vec<ChannelId> = channels_id.iter().map(|a| a.id).collect();
		let vec_string: Vec<String> = vec_channel_id.iter().map(|id| id.to_string()).collect();
		let user_id = command_interaction.user.id.to_string();

		let condition = Condition::all()
			.add(message::Column::UserId.eq(user_id))
			.add(message::Column::ChannelId.is_in(vec_string));

        let db_connection = bot_data.db_connection.clone();

		let message = DatabaseMessage::find()
            .filter(condition)
		    .all(&*db_connection)
	        .await?;

		Err(anyhow!("Not implemented yet"))
	}
);
