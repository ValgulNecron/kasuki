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
					.table(MessageSummary::Table)
					.if_not_exists()
					.col(string(MessageSummary::UserId))
					.col(string(MessageSummary::ChannelId))
					.col(big_integer(MessageSummary::MessageCount).default(0i64))
					.primary_key(
						Index::create()
							.col(MessageSummary::UserId)
							.col(MessageSummary::ChannelId),
					)
					.foreign_key(
						ForeignKey::create()
							.name("FK_message_summary_user")
							.to(UserData::Table, UserData::UserId)
							.from(MessageSummary::Table, MessageSummary::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(MessageSummary::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
#[sea_orm(iden = "message_summary")]
pub enum MessageSummary {
	Table,
	#[sea_orm(iden = "user_id")]
	UserId,
	#[sea_orm(iden = "channel_id")]
	ChannelId,
	#[sea_orm(iden = "message_count")]
	MessageCount,
}
