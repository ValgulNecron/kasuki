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
					.table(GuildLang::Table)
					.if_not_exists()
					.col(string(GuildLang::GuildId))
					.primary_key(Index::create().col(GuildLang::GuildId))
					.col(string(GuildLang::Lang))
					.col(timestamp(GuildLang::UpdatedAt).default(Expr::current_timestamp()))
					.foreign_key(
						ForeignKey::create()
							.name("FK_server_lang")
							.to(GuildData::Table, GuildData::GuildId)
							.from(GuildLang::Table, GuildLang::GuildId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(GuildLang::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum GuildLang {
	Table,
	GuildId,
	Lang,
	UpdatedAt,
}
