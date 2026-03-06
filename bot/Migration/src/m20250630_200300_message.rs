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
					.table(Message::Table)
					.if_not_exists()
					.col(string(Message::Id).primary_key())
					.col(string(Message::UserId))
					.col(text(Message::Data))
					.col(integer(Message::ChatLength))
					.col(string(Message::ChannelId))
					.foreign_key(
						ForeignKey::create()
							.name("FK_message_user")
							.to(UserData::Table, UserData::UserId)
							.from(Message::Table, Message::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Message::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum Message {
	Table,
	Id,
	UserId,
	Data,
	ChatLength,
	ChannelId,
}
