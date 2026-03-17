use crate::event_handler::{BotData, Handler};
use crate::handlers::user_db::add_user_data_to_db;
use serenity::all::Presence;
use serenity::prelude::Context as SerenityContext;
use tracing::{trace, warn};

impl Handler {
	pub(crate) async fn presence_update(
		&self, ctx: &SerenityContext, _old_data: Option<Presence>, new_data: Presence,
	) {
		let bot_data = ctx.data::<BotData>().clone();
		let user_id = new_data.user.id;
		trace!(user_id = %user_id, "Presence update received");

		let user = new_data.user.to_user();

		if let Some(user) = user {
			if let Err(e) = add_user_data_to_db(user, bot_data.db_connection.clone()).await {
				warn!(
					user_id = %user_id,
					error = ?e,
					"Failed to insert user data from presence update into database"
				);
			}
		}
	}
}
