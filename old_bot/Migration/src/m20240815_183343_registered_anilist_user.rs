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
					.table(RegisteredUser::Table)
					.if_not_exists()
					.col(string(RegisteredUser::UserId))
					.primary_key(Index::create().col(RegisteredUser::UserId))
					.col(integer(RegisteredUser::AnilistId))
					.col(timestamp(RegisteredUser::RegisteredAt).default(Expr::current_timestamp()))
					.foreign_key(
						ForeignKey::create()
							.name("FK_user_registered_user")
							.to(UserData::Table, UserData::UserId)
							.from(RegisteredUser::Table, RegisteredUser::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(RegisteredUser::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum RegisteredUser {
	Table,
	UserId,
	AnilistId,
	RegisteredAt,
}
