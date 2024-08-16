use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PingHistory::Table)
                    .if_not_exists()
                    .col(string(PingHistory::ShardId))
                    .primary_key(
                        Index::create().col(PingHistory::ShardId)
                    )
                    .col(timestamp(PingHistory::Timestamp))
                    .col(string(PingHistory::Latency))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PingHistory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PingHistory {
    Table,
    ShardId,
    Timestamp,
    Latency,
}
