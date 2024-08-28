use crate::m20240815_231531_guild_data::GuildData;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ActivityData::Table)
                    .if_not_exists()
                    .col(integer(ActivityData::AnimeId))
                    .col(string(ActivityData::ServerId))
                    .primary_key(
                        Index::create()
                            .col(ActivityData::AnimeId)
                            .col(ActivityData::ServerId),
                    )
                    .col(integer(ActivityData::Episode))
                    .col(string(ActivityData::Webhook))
                    .col(string(ActivityData::Name))
                    .col(string(ActivityData::Image))
                    .col(integer(ActivityData::Delay))
                    .col(timestamp(ActivityData::Timestamp).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_server_activity")
                            .to(GuildData::Table, GuildData::GuildId)
                            .from(ActivityData::Table, ActivityData::ServerId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ActivityData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ActivityData {
    Table,
    AnimeId,
    ServerId,
    Timestamp,
    Webhook,
    Episode,
    Name,
    Delay,
    Image,
}
