use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GuildLang::Table)
                    .if_not_exists()
                    .col(string(GuildLang::GuildId))
                    .primary_key(
                        Index::create().col(GuildLang::GuildId)
                    )
                    .col(string(GuildLang::Lang))
                    .col(timestamp(GuildLang::UpdatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildLang::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GuildLang {
    Table,
    GuildId,
    Lang,
    UpdatedAt,
}
