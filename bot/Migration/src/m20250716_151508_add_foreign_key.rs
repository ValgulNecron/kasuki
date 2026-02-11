use crate::m20240815_180000_guild_data::GuildData;
use crate::m20240815_180201_user_data::UserData;
use crate::m20250712_120918_create_inventory::UserInventory;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Clean up orphaned rows that would violate the new foreign key constraints
		let db = manager.get_connection();
		db.execute_unprepared(
			"DELETE FROM user_inventory WHERE server_id NOT IN (SELECT guild_id FROM guild_data)",
		)
		.await?;
		db.execute_unprepared(
			"DELETE FROM user_inventory WHERE user_id NOT IN (SELECT user_id FROM user_data)",
		)
		.await?;
		// Remove duplicate (user_id, server_id, item_id) rows, keeping one per combination
		db.execute_unprepared(
			"DELETE FROM user_inventory a USING user_inventory b \
			 WHERE a.ctid < b.ctid \
			 AND a.user_id = b.user_id \
			 AND a.server_id = b.server_id \
			 AND a.item_id = b.item_id",
		)
		.await?;

		manager
			.alter_table(
				TableAlterStatement::new()
					.table(UserInventory::Table)
					.add_foreign_key(
						TableForeignKey::new()
							.name("fk-inventory-server_id")
							.from_tbl(UserInventory::Table)
							.from_col(UserInventory::ServerId)
							.to_tbl(GuildData::Table)
							.to_col(GuildData::GuildId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.add_foreign_key(
						TableForeignKey::new()
							.name("fk-inventory-user_id")
							.from_tbl(UserInventory::Table)
							.from_col(UserInventory::UserId)
							.to_tbl(UserData::Table)
							.to_col(UserData::UserId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				TableAlterStatement::new()
					.table(UserInventory::Table)
					.drop_foreign_key(Alias::new("fk-inventory-server_id"))
					.drop_foreign_key(Alias::new("fk-inventory-user_id"))
					.to_owned(),
			)
			.await
	}
}
