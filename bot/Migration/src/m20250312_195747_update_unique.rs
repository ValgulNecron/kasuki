use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250312_add_anilist_id_unique_constraint"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table("registered_user")
                    .name("idx_registered_user_anilist_id_unique")
                    .col("anilist_id")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("guild_data")
                    .name("idx_guild_data_guild_name_unique")
                    .col("guild_name")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("guild_data")
                    .name("idx_guild_data_updated_at_unique")
                    .col("updated_at")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("guild_subscription")
                    .name("idx_guild_subscription_guild_id_unique")
                    .col("guild_id")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("guild_subscription")
                    .name("idx_guild_subscription_sku_id_unique")
                    .col("sku_id")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("user_subscription")
                    .name("idx_user_subscription_user_id_unique")
                    .col("user_id")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("user_subscription")
                    .name("idx_user_subscription_sku_id_unique")
                    .col("sku_id")
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table("user_subscription")
                    .name("idx_user_subscription_sku_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table("user_subscription")
                    .name("idx_user_subscription_user_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table("guild_subscription")
                    .name("idx_guild_subscription_sku_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table("guild_subscription")
                    .name("idx_guild_subscription_guild_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table("guild_data")
                    .name("idx_guild_data_updated_at_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table("guild_data")
                    .name("idx_guild_data_guild_name_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table("registered_user")
                    .name("idx_registered_user_anilist_id_unique")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}