use anyhow::anyhow;
use sea_orm::EntityTrait;use sea_orm::QueryFilter;
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::management::give_premium_sub::GivePremiumSubCommand;
use crate::database::{message, vocal};
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
			.add(message::Column::UserId.eq(user_id.clone()))
			.add(message::Column::ChannelId.is_in(vec_string.clone()));

        let db_connection = bot_data.db_connection.clone();

		let messages = DatabaseMessage::find()
            .filter(condition)
		    .all(&*db_connection)
	        .await?;

		let total_message = messages.len()  as i128;
		let mut total_message_len: i128 = 0;
		for message in messages {
			total_message_len += message.chat_length  as i128;
		}

		let condition = Condition::all()
			.add(vocal::Column::UserId.eq(user_id))
			.add(vocal::Column::ChannelId.is_in(vec_string));
		let vocals = DatabaseVocal::find()
		.filter(condition)
		.all(&*db_connection)
		.await?;

		let total_vocal = vocals.len() as i128;
		let mut total_vocal_len: i128 = 0;
		for vocal in vocals {
			total_vocal_len += vocal.duration  as i128;
		}

		let xp: i128 = total_message_len *2 + total_message + total_vocal_len*2 + total_vocal;

		Err(anyhow!("Not implemented yet"))
	}
);

fn get_level(xp: i32) -> i32 {
	match xp {
		0..=1000 => 1,
		1001..=3000 => 2,
		3001..=5000 => 3,
		5001..=8000 => 4,
		8001..=13000 => 5,
		13001..=21000 => 6,
		21001..=34000 => 7,
		34001..=55000 => 8,
		55001..=89000 => 9,
		_ => 10,
	}
}
