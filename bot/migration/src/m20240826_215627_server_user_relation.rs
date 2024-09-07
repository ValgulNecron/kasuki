use crate::m20240815_180000_guild_data::GuildData;
use crate::m20240815_180001_user_data::UserData;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServerUserRelation::Table)
                    .if_not_exists()
                    .col(string(ServerUserRelation::UserId))
                    .col(string(ServerUserRelation::GuildId))
                    .primary_key(
                        Index::create()
                            .col(ServerUserRelation::UserId)
                            .col(ServerUserRelation::GuildId),
                    )
                    .foreign_key(
                        &mut ForeignKey::create()
                            .name("FK_user_relation")
                            .to(UserData::Table, UserData::UserId)
                            .from(ServerUserRelation::Table, ServerUserRelation::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        &mut ForeignKey::create()
                            .name("FK_server_relation")
                            .to(GuildData::Table, GuildData::GuildId)
                            .from(ServerUserRelation::Table, ServerUserRelation::GuildId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServerUserRelation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ServerUserRelation {
    Table,
    UserId,
    GuildId,
}
