use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GuildData::Table)
                    .if_not_exists()
                    .col(string(GuildData::GuildId))
                    .primary_key(
                        Index::create().col(GuildData::GuildId)
                    )
                    .col(string(GuildData::GuildName))
                    .col(timestamp(GuildData::UpdatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GuildData {
    Table,
    GuildId,
    GuildName,
    UpdatedAt,
}
