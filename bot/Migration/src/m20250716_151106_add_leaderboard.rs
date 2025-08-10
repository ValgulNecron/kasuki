use crate::m20240815_180000_guild_data::GuildData;
use crate::m20240815_180201_user_data::UserData;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(LeaderBoard::Table)
					.if_not_exists()
					.col(ColumnDef::new(LeaderBoard::UserId).string().not_null())
					.col(
						ColumnDef::new(LeaderBoard::MinigameType)
							.string()
							.not_null(),
					)
					.col(ColumnDef::new(LeaderBoard::ServerId).string().not_null())
					.col(ColumnDef::new(LeaderBoard::Points).integer().not_null())
					.primary_key(
						Index::create()
							.col(LeaderBoard::MinigameType)
							.col(LeaderBoard::ServerId)
							.col(LeaderBoard::UserId),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk-leaderboard-user_id")
							.from(LeaderBoard::Table, LeaderBoard::UserId)
							.to(UserData::Table, UserData::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk-leaderboard-server_id")
							.from(LeaderBoard::Table, LeaderBoard::ServerId)
							.to(GuildData::Table, GuildData::GuildId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(LeaderBoard::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum LeaderBoard {
	Table,
	ServerId,
	UserId,
	Points,
	MinigameType,
}
