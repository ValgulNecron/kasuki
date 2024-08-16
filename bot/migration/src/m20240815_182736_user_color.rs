use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(UserColor::Table)
                    .if_not_exists()
                    .col(string(UserColor::UserId))
                    .primary_key(
                        Index::create().col(UserColor::UserId)
                    )
                    .col(string(UserColor::Color))
                    .col(string(UserColor::Images))
                    .col(string(UserColor::ProfilePictureUrl))
                    .col(timestamp(UserColor::CalculatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserColor::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserColor {
    Table,
    UserId,
    Color,
    ProfilePictureUrl,
    Images,
    CalculatedAt,
}
