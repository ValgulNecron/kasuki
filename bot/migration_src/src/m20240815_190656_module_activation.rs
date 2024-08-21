use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ModuleActivation::Table)
                    .if_not_exists()
                    .col(string(ModuleActivation::GuildId))
                    .primary_key(
                        Index::create().col(ModuleActivation::GuildId)
                    )
                    .col(boolean(ModuleActivation::AIModule).default(true))
                    .col(boolean(ModuleActivation::AnilistModule).default(true))
                    .col(boolean(ModuleActivation::GameModule).default(true))
                    .col(boolean(ModuleActivation::NewMembersModule).default(false))
                    .col(boolean(ModuleActivation::AnimeModule).default(true))
                    .col(boolean(ModuleActivation::VnModule).default(true))
                    .col(timestamp(ModuleActivation::UpdatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ModuleActivation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ModuleActivation {
    Table,
    GuildId,
    AIModule,
    AnilistModule,
    GameModule,
    NewMembersModule,
    AnimeModule,
    VnModule,
    UpdatedAt,
}
