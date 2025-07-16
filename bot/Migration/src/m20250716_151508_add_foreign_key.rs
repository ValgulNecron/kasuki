use crate::m20240815_180000_guild_data::GuildData;
use crate::m20240815_180201_user_data::UserData;
use crate::m20250712_120918_create_inventory::UserInventory;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				TableAlterStatement::new()
					.table(UserInventory::Table)
					.add_foreign_key(
						TableForeignKey::new()
							.from_tbl(GuildData::Table)
							.from_col(GuildData::GuildId)
							.to_tbl(UserInventory::Table)
							.to_col(UserInventory::ServerId)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.add_foreign_key(
						TableForeignKey::new()
							.from_tbl(UserData::Table)
							.from_col(UserData::UserId)
							.to_tbl(UserInventory::Table)
							.to_col(UserInventory::UserId)
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
					.drop_foreign_key(Alias::new("fk-guild_data-guild_id"))
					.drop_foreign_key(Alias::new("fk-user_data-user_id"))
					.to_owned(),
			)
			.await
	}
}
