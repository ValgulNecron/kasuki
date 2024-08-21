use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(KillSwitch::Table)
                    .if_not_exists()
                    .col(string(KillSwitch::GuildId))
                    .primary_key(
                        Index::create().col(KillSwitch::GuildId)
                    )
                    .col(boolean(KillSwitch::AIModule).default(true))
                    .col(boolean(KillSwitch::AnilistModule).default(true))
                    .col(boolean(KillSwitch::GameModule).default(true))
                    .col(boolean(KillSwitch::NewMembersModule).default(false))
                    .col(boolean(KillSwitch::AnimeModule).default(true))
                    .col(boolean(KillSwitch::VnModule).default(true))
                    .col(timestamp(KillSwitch::UpdatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KillSwitch::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum KillSwitch {
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
