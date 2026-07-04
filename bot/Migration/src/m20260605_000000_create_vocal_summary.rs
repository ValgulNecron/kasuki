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
					.table(VocalSummary::Table)
					.if_not_exists()
					.col(string(VocalSummary::UserId))
					.col(string(VocalSummary::ChannelId))
					.col(big_integer(VocalSummary::SessionCount).default(0i64))
					.col(big_integer(VocalSummary::DurationTotalSeconds).default(0i64))
					.primary_key(
						Index::create()
							.col(VocalSummary::UserId)
							.col(VocalSummary::ChannelId),
					)
					.foreign_key(
						ForeignKey::create()
							.name("FK_vocal_summary_user")
							.to(UserData::Table, UserData::UserId)
							.from(VocalSummary::Table, VocalSummary::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(VocalSummary::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
#[sea_orm(iden = "vocal_summary")]
pub enum VocalSummary {
	Table,
	#[sea_orm(iden = "user_id")]
	UserId,
	#[sea_orm(iden = "channel_id")]
	ChannelId,
	#[sea_orm(iden = "session_count")]
	SessionCount,
	#[sea_orm(iden = "duration_total_seconds")]
	DurationTotalSeconds,
}
