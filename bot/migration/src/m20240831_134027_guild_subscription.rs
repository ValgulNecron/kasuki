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
                    .table(GuildSubscription::Table)
                    .if_not_exists()
                    .col(string(GuildSubscription::GuildId))
                    .col(string(GuildSubscription::EntitlementId))
                    .primary_key(
                        Index::create()
                            .col(GuildSubscription::GuildId)
                            .col(GuildSubscription::SkuId),
                    )
                    .col(string(GuildSubscription::SkuId))
                    .col(timestamp(GuildSubscription::CreatedAt))
                    .col(timestamp(GuildSubscription::UpdatedAt))
                    .col(timestamp(GuildSubscription::ExpiredAt))
                    .foreign_key(
                        &mut ForeignKey::create()
                            .name("FK_guild_subscription")
                            .to(UserData::Table, UserData::UserId)
                            .from(GuildSubscription::Table, GuildSubscription::GuildId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildSubscription::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GuildSubscription {
    Table,
    GuildId,
    EntitlementId,
    SkuId,
    CreatedAt,
    UpdatedAt,
    ExpiredAt,
}