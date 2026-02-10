use crate::m20240815_180201_user_data::UserData;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				TableAlterStatement::new()
					.table(UserData::Table)
					.drop_column(UserData::Banner)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				TableAlterStatement::new()
					.table(UserData::Table)
					.add_column_if_not_exists(
						ColumnDef::new(UserData::Banner)
							.string()
							.not_null()
							.default(""),
					)
					.to_owned(),
			)
			.await
	}
}
