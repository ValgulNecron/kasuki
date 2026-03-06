use crate::event_handler::{BotData, Handler};
use crate::handlers::user_db::add_user_data_to_db;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::Message;
use serenity::prelude::Context as SerenityContext;
use shared::database::prelude::Message as DatabaseMessage;
use tracing::{trace, warn};

impl Handler {
	pub(crate) async fn new_message(&self, ctx: SerenityContext, message: Message) {
		let bot_data = ctx.data::<BotData>().clone();
		let user_blacklist = bot_data.user_blacklist.clone();
		let read_guard = user_blacklist.read().await;
		let user_id = message.author.id;

		if read_guard.contains(&user_id.to_string()) {
			return;
		}
		trace!(
			message_id = %message.id,
			user_id = %user_id,
			"New message received"
		);

		let db_connection = bot_data.db_connection.clone();

		// Ensure user exists before inserting message (FK constraint)
		if let Err(e) = add_user_data_to_db(message.author.clone(), db_connection.clone()).await {
			warn!(
				user_id = %user_id,
				error = ?e,
				"Failed to insert user data from message into database"
			);
		}

		let message_id = message.id.to_string();
		let data = message.content.to_string();
		let length = data.len();
		let channel_id = message.channel_id.to_string();

		let active_message = shared::database::message::ActiveModel {
			id: Set(message_id),
			user_id: Set(user_id.to_string()),
			data: Set(data),
			chat_length: Set(length as i32),
			channel_id: Set(channel_id),
		};

		if let Err(e) = DatabaseMessage::insert(active_message)
			.exec(&*db_connection)
			.await
		{
			warn!(
				message_id = %message.id,
				user_id = %user_id,
				error = %e,
				"Failed to insert message into database"
			);
		}
	}
}
