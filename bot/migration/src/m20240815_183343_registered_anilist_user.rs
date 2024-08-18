use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RegisteredUser::Table)
                    .if_not_exists()
                    .col(string(RegisteredUser::UserId))
                    .primary_key(Index::create().col(RegisteredUser::UserId))
                    .col(integer(RegisteredUser::AnilistId))
                    .col(timestamp(RegisteredUser::RegisteredAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RegisteredUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RegisteredUser {
    Table,
    UserId,
    AnilistId,
    RegisteredAt,
}
