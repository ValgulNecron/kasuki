use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(CommandList::Table)
					.if_not_exists()
					.col(string(CommandList::CommandName))
					.primary_key(Index::create().col(CommandList::CommandName))
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(CommandList::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum CommandList {
	Table,
	CommandName,
}
