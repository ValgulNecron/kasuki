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
			.get_connection()
			.execute_unprepared(
				"ALTER TABLE command_usage ADD PRIMARY KEY (command, \"user\", use_time)",
			)
			.await
			.map(|_| ())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();
		db.execute_unprepared("ALTER TABLE command_usage DROP CONSTRAINT command_usage_pkey")
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(CommandUsage::Table)
					.drop_column(CommandUsage::UseTime)
					.to_owned(),
			)
			.await?;

		db.execute_unprepared("ALTER TABLE command_usage ADD PRIMARY KEY (command, \"user\")")
			.await
			.map(|_| ())
	}
}

#[derive(DeriveIden)]
enum CommandUsage {
	Table,
	Command,
	User,
	UseTime,
}
