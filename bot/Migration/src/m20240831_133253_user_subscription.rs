use crate::m20240815_180201_user_data::UserData;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(UserSubscription::Table)
					.if_not_exists()
					.col(string(UserSubscription::UserId))
					.col(string(UserSubscription::EntitlementId))
					.primary_key(
						Index::create()
							.col(UserSubscription::UserId)
							.col(UserSubscription::SkuId),
					)
					.col(string(UserSubscription::SkuId))
					.col(timestamp(UserSubscription::CreatedAt))
					.col(timestamp(UserSubscription::UpdatedAt))
					.col(timestamp(UserSubscription::ExpiredAt))
					.foreign_key(
						ForeignKey::create()
							.name("FK_user_subscription")
							.to(UserData::Table, UserData::UserId)
							.from(UserSubscription::Table, UserSubscription::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(UserSubscription::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum UserSubscription {
	Table,
	UserId,
	EntitlementId,
	SkuId,
	CreatedAt,
	UpdatedAt,
	ExpiredAt,
}
