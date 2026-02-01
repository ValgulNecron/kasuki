use crate::m20240815_180201_user_data::UserData;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Vocal::Table)
                    .if_not_exists()
                    .col(string(Vocal::Id).primary_key())
                    .col(string(Vocal::UserId))
                    .col(timestamp(Vocal::Start))
                    .col(timestamp(Vocal::End))
                    .col(integer(Vocal::Duration))
                    .col(string(Vocal::ChannelId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_vocal_user")
                            .to(UserData::Table, UserData::UserId)
                            .from(Vocal::Table, Vocal::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vocal::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Vocal {
    Table,
    Id,
    UserId,
    Start,
    End,
    Duration,
    ChannelId,
}
