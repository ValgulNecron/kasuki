use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
	fn name(&self) -> &str {
		"m20260317_add_channel_id_to_activity"
	}
}

#[derive(DeriveIden)]
enum ActivityData {
	Table,
	ChannelId,
	Webhook,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Add channel_id column (nullable) so API can create activities without a webhook
		manager
			.alter_table(
				Table::alter()
					.table(ActivityData::Table)
					.add_column(
						ColumnDef::new(ActivityData::ChannelId)
							.string()
							.null()
							.to_owned(),
					)
					.to_owned(),
			)
			.await?;

		// Make webhook column nullable so API-created activities can omit it
		manager
			.alter_table(
				Table::alter()
					.table(ActivityData::Table)
					.modify_column(
						ColumnDef::new(ActivityData::Webhook)
							.string()
							.null()
							.to_owned(),
					)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(ActivityData::Table)
					.modify_column(
						ColumnDef::new(ActivityData::Webhook)
							.string()
							.not_null()
							.to_owned(),
					)
					.to_owned(),
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(ActivityData::Table)
					.drop_column(ActivityData::ChannelId)
					.to_owned(),
			)
			.await?;

		Ok(())
	}
}
