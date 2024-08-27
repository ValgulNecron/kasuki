use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserData::Table)
                    .if_not_exists()
                    .col(string(UserData::UserId))
                    .primary_key(Index::create().col(UserData::UserId))
                    .col(string(UserData::Username))
                    .col(timestamp(UserData::AddedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum UserData {
    Table,
    UserId,
    Username,
    AddedAt,
}
