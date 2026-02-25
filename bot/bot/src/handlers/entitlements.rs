use crate::event_handler::{BotData, Handler};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::Entitlement;
use serenity::prelude::Context as SerenityContext;
use shared::database::prelude::{GuildSubscription, UserSubscription};
use std::sync::Arc;
use sea_orm::DatabaseConnection;
use tracing::{info, warn};

impl Handler {
	pub(crate) async fn entitlement_create(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		info!(
			entitlement_id = %entitlement.id,
			"New entitlement created"
		);
		insert_subscription(entitlement, connection).await;
	}

	pub(crate) async fn entitlement_update(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		info!(
			entitlement_id = %entitlement.id,
			"Entitlement updated"
		);
		insert_subscription(entitlement, connection).await;
	}

	pub(crate) async fn entitlement_delete(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		info!(
			entitlement_id = %entitlement.id,
			"Entitlement deleted"
		);
		let (guild_id, user_id) = (entitlement.guild_id, entitlement.user_id);
		let sku_id = entitlement.sku_id.to_string();

		if let Some(guild_id) = guild_id {
			let guild_id = guild_id.to_string();
			if let Err(e) = GuildSubscription::delete_by_id((guild_id.clone(), sku_id.clone()))
				.exec(&*connection)
				.await
			{
				warn!(
					guild_id = %guild_id,
					sku_id = %sku_id,
					error = %e,
					"Failed to delete guild subscription"
				);
			}
		} else if let Some(user_id) = user_id {
			let user_id = user_id.to_string();
			if let Err(e) = UserSubscription::delete_by_id((user_id.clone(), sku_id.clone()))
				.exec(&*connection)
				.await
			{
				warn!(
					user_id = %user_id,
					sku_id = %sku_id,
					error = %e,
					"Failed to delete user subscription"
				);
			}
		}
	}
}

async fn insert_subscription(entitlement: Entitlement, connection: Arc<DatabaseConnection>) {
	info!(
		entitlement_id = %entitlement.id,
		"Inserting or updating subscription"
	);
	match (entitlement.guild_id, entitlement.user_id) {
		(Some(guild_id), None) => {
			insert_guild_subscription(entitlement, guild_id.to_string(), connection).await
		},
		(None, Some(user_id)) => {
			insert_user_subscription(entitlement, user_id.to_string(), connection).await
		},
		_ => {},
	}
}

async fn insert_guild_subscription(
	entitlement: Entitlement, guild_id: String, connection: Arc<DatabaseConnection>,
) {
	let model = shared::database::guild_subscription::ActiveModel {
		guild_id: Set(guild_id.clone()),
		entitlement_id: Set(entitlement.id.to_string()),
		sku_id: Set(entitlement.sku_id.to_string()),
		created_at: Set(entitlement.starts_at.unwrap_or_default().naive_utc()),
		updated_at: Default::default(),
		expired_at: Default::default(),
	};
	if let Err(e) = GuildSubscription::insert(model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::columns([
				shared::database::guild_subscription::Column::GuildId,
				shared::database::guild_subscription::Column::SkuId,
			])
			.update_columns([
				shared::database::guild_subscription::Column::EntitlementId,
				shared::database::guild_subscription::Column::ExpiredAt,
				shared::database::guild_subscription::Column::UpdatedAt,
			])
			.to_owned(),
		)
		.exec(&*connection)
		.await
	{
		warn!(
			guild_id = %guild_id,
			sku_id = %entitlement.sku_id,
			error = %e,
			"Failed to insert or update guild subscription"
		);
	}
}

async fn insert_user_subscription(
	entitlement: Entitlement, user_id: String, connection: Arc<DatabaseConnection>,
) {
	let model = shared::database::user_subscription::ActiveModel {
		user_id: Set(user_id.clone()),
		entitlement_id: Set(entitlement.id.to_string()),
		sku_id: Set(entitlement.sku_id.to_string()),
		created_at: Set(entitlement.starts_at.unwrap_or_default().naive_utc()),
		updated_at: Default::default(),
		expired_at: Default::default(),
	};
	if let Err(e) = UserSubscription::insert(model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::columns([
				shared::database::user_subscription::Column::UserId,
				shared::database::user_subscription::Column::SkuId,
			])
			.update_columns([
				shared::database::user_subscription::Column::EntitlementId,
				shared::database::user_subscription::Column::ExpiredAt,
				shared::database::user_subscription::Column::UpdatedAt,
			])
			.to_owned(),
		)
		.exec(&*connection)
		.await
	{
		warn!(
			user_id = %user_id,
			sku_id = %entitlement.sku_id,
			error = %e,
			"Failed to insert or update user subscription"
		);
	}
}
