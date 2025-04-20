use crate::m20240815_180201_user_data::UserData;
use crate::m20241221_192311_command_list::CommandList;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CommandUsage::Table)
                    .if_not_exists()
                    .col(string(CommandUsage::Command))
                    .col(string(CommandUsage::User))
                    .primary_key(
                        Index::create()
                            .col(CommandUsage::Command)
                            .col(CommandUsage::User),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_user_relation")
                            .to(UserData::Table, UserData::UserId)
                            .from(CommandUsage::Table, CommandUsage::User)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_server_relation")
                            .to(CommandList::Table, CommandList::CommandName)
                            .from(CommandUsage::Table, CommandUsage::Command)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CommandUsage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CommandUsage {
    Table,
    Command,
    User,
}
