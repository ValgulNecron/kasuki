use anyhow::Result;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serenity::all::User;
use shared::database::prelude::UserData;
use std::sync::Arc;
use tracing::{trace, warn};

pub async fn add_user_data_to_db(user: User, connection: Arc<DatabaseConnection>) -> Result<()> {
	trace!(user_id = %user.id, "Adding user data to database");
	let model = shared::database::user_data::ActiveModel {
		user_id: Set(user.id.to_string()),
		username: Set(user.name.to_string()),
		added_at: Set(Utc::now().naive_utc()),
		is_bot: Set(user.bot()),
	};
	// Upsert: insert new user or refresh username/bot-flag on conflict (user may have renamed)
	if let Err(e) = UserData::insert(model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::column(shared::database::user_data::Column::UserId)
				.update_columns([
					shared::database::user_data::Column::Username,
					shared::database::user_data::Column::IsBot,
				])
				.to_owned(),
		)
		.exec(&*connection)
		.await
	{
		warn!(
			user_id = %user.id,
			error = %e,
			"Failed to insert or update user data in database"
		);
		Err(e.into())
	} else {
		Ok(())
	}
}
