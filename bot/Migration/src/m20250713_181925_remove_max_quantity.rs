use crate::m20250712_120900_create_item::Item;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				TableAlterStatement::new()
					.table(Item::Table)
					.drop_column(Item::MaxNumber)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				TableAlterStatement::new()
					.table(Item::Table)
					.add_column_if_not_exists(ColumnDef::new(Item::MaxNumber).integer().not_null())
					.to_owned(),
			)
			.await
	}
}
