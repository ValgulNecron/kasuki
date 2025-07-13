use sea_orm_migration::{prelude::*, schema::*};
use crate::m20240815_190656_module_activation::ModuleActivation;
use crate::m20240815_231355_kill_switch::KillSwitch;
use crate::m20250712_120900_create_item::Item;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserInventory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserInventory::ItemId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInventory::UserId).string().not_null())
                    .col(ColumnDef::new(UserInventory::ServerId).string().not_null())
                    .col(ColumnDef::new(UserInventory::Size).integer().not_null())
                    .col(ColumnDef::new(UserInventory::Rarity).integer().not_null())
                    .col(ColumnDef::new(UserInventory::ItemXPBoost).float().not_null())
                    .col(ColumnDef::new(UserInventory::Id).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(UserInventory::Id)
                            .col(UserInventory::ItemId)
                            .col(UserInventory::UserId)
                            .col(UserInventory::ServerId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserInventory::Table, UserInventory::ItemId)
                            .to(Item::Table, Item::ItemId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserInventory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum UserInventory {
    Table,
    Id,
    ItemId,
    UserId,
    ServerId,
    Size,
    Rarity,
    ItemXPBoost,
}
