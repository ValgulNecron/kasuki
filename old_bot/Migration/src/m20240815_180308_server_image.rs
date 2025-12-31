use crate::m20240815_180000_guild_data::GuildData;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(ServerImage::Table)
					.if_not_exists()
					.col(string(ServerImage::ServerId))
					.col(string(ServerImage::ImageType))
					.primary_key(
						Index::create()
							.col(ServerImage::ServerId)
							.col(ServerImage::ImageType),
					)
					.col(string(ServerImage::ServerName))
					.col(string(ServerImage::Image))
					.col(string(ServerImage::ImageUrl))
					.col(timestamp(ServerImage::GeneratedAt).default(Expr::current_timestamp()))
					.foreign_key(
						ForeignKey::create()
							.name("FK_server_image")
							.to(GuildData::Table, GuildData::GuildId)
							.from(ServerImage::Table, ServerImage::ServerId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(ServerImage::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum ServerImage {
	Table,
	ServerId,
	ImageType,
	Image,
	ImageUrl,
	ServerName,
	GeneratedAt,
}
