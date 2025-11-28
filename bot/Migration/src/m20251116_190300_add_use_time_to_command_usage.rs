use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(CommandUsage::Table)
					.add_column(
						ColumnDef::new(CommandUsage::UseTime)
							.timestamp()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.to_owned(),
			)
			.await?;

		// The default primary key name for postgres is <table_name>_pkey
		let query = "ALTER TABLE command_usage DROP CONSTRAINT command_usage_pkey";
		manager
			.get_connection()
			.execute_unprepared(query)
			.await
			.map(|_| ())?;

		manager
			.create_index(
				Index::create()
                    .name("command_usage_pkey") // Name it so we can drop it easily
                    .table(CommandUsage::Table)
                    .col(CommandUsage::Command)
                    .col(CommandUsage::User)
                    .col(CommandUsage::UseTime)
                    .primary()
                    .to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(
				Index::drop()
					.name("command_usage_pkey")
					.table(CommandUsage::Table)
					.to_owned(),
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(CommandUsage::Table)
					.drop_column(CommandUsage::UseTime)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("command_usage_pkey")
					.table(CommandUsage::Table)
					.col(CommandUsage::Command)
					.col(CommandUsage::User)
					.primary()
					.to_owned(),
			)
			.await
	}
}

#[derive(DeriveIden)]
enum CommandUsage {
	Table,
	Command,
	User,
	UseTime,
}
