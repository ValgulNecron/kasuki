use crate::m20240815_180000_guild_data::GuildData;
use crate::m20240831_133253_user_subscription::UserSubscription;
use crate::m20240831_134027_guild_subscription::GuildSubscription;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
	fn name(&self) -> &str {
		"m20260307_drop_bad_unique_constraints"
	}
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// guild_name is not unique — multiple guilds can share a name
		manager
			.drop_index(
				Index::drop()
					.table(GuildData::Table)
					.name("idx_guild_data_guild_name_unique")
					.to_owned(),
			)
			.await?;

		// updated_at is not unique — multiple guilds can update at the same time
		manager
			.drop_index(
				Index::drop()
					.table(GuildData::Table)
					.name("idx_guild_data_updated_at_unique")
					.to_owned(),
			)
			.await?;

		// guild_id is part of composite PK (guild_id, sku_id) — not unique alone
		manager
			.drop_index(
				Index::drop()
					.table(GuildSubscription::Table)
					.name("idx_guild_subscription_guild_id_unique")
					.to_owned(),
			)
			.await?;

		// sku_id can be shared across guilds
		manager
			.drop_index(
				Index::drop()
					.table(GuildSubscription::Table)
					.name("idx_guild_subscription_sku_id_unique")
					.to_owned(),
			)
			.await?;

		// user_id is part of composite PK (user_id, sku_id) — not unique alone
		manager
			.drop_index(
				Index::drop()
					.table(UserSubscription::Table)
					.name("idx_user_subscription_user_id_unique")
					.to_owned(),
			)
			.await?;

		// sku_id can be shared across users
		manager
			.drop_index(
				Index::drop()
					.table(UserSubscription::Table)
					.name("idx_user_subscription_sku_id_unique")
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Re-create the constraints (even though they were wrong)
		manager
			.create_index(
				Index::create()
					.table(GuildData::Table)
					.name("idx_guild_data_guild_name_unique")
					.col(GuildData::GuildName)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.table(GuildData::Table)
					.name("idx_guild_data_updated_at_unique")
					.col(GuildData::UpdatedAt)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.table(GuildSubscription::Table)
					.name("idx_guild_subscription_guild_id_unique")
					.col(GuildSubscription::GuildId)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.table(GuildSubscription::Table)
					.name("idx_guild_subscription_sku_id_unique")
					.col(GuildSubscription::SkuId)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.table(UserSubscription::Table)
					.name("idx_user_subscription_user_id_unique")
					.col(UserSubscription::UserId)
					.unique()
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.table(UserSubscription::Table)
					.name("idx_user_subscription_sku_id_unique")
					.col(UserSubscription::SkuId)
					.unique()
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}
