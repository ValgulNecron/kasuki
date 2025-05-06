use crate::m20240815_180000_guild_data::GuildData;
// Import the enum identifier types from your files - adjust paths as needed
// You'll need to import these enum types from where they are defined
use crate::m20240815_183343_registered_anilist_user::RegisteredUser;
use crate::m20240831_133253_user_subscription::UserSubscription;
use crate::m20240831_134027_guild_subscription::GuildSubscription;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250312_add_multiple_unique_constraints"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add unique constraint to RegisteredUser.anilist_id
        manager
            .create_index(
                Index::create()
                    .table(RegisteredUser::Table)
                    .name("idx_registered_user_anilist_id_unique")
                    .col(RegisteredUser::AnilistId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add unique constraint to GuildData.guild_name
        manager
            .create_index(
                Index::create()
                    .table(GuildData::Table)
                    .name("idx_guild_data_guild_name_unique")
                    .col(GuildData::GuildName)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add unique constraint to GuildData.updated_at
        manager
            .create_index(
                Index::create()
                    .table(GuildData::Table)
                    .name("idx_guild_data_updated_at_unique")
                    .col(GuildData::UpdatedAt)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add unique constraint to GuildSubscription.guild_id
        manager
            .create_index(
                Index::create()
                    .table(GuildSubscription::Table)
                    .name("idx_guild_subscription_guild_id_unique")
                    .col(GuildSubscription::GuildId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add unique constraint to GuildSubscription.sku_id
        manager
            .create_index(
                Index::create()
                    .table(GuildSubscription::Table)
                    .name("idx_guild_subscription_sku_id_unique")
                    .col(GuildSubscription::SkuId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add unique constraint to UserSubscription.user_id
        manager
            .create_index(
                Index::create()
                    .table(UserSubscription::Table)
                    .name("idx_user_subscription_user_id_unique")
                    .col(UserSubscription::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add unique constraint to UserSubscription.sku_id
        manager
            .create_index(
                Index::create()
                    .table(UserSubscription::Table)
                    .name("idx_user_subscription_sku_id_unique")
                    .col(UserSubscription::SkuId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the unique indices in reverse order
        manager
            .drop_index(
                Index::drop()
                    .table(UserSubscription::Table)
                    .name("idx_user_subscription_sku_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(UserSubscription::Table)
                    .name("idx_user_subscription_user_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(GuildSubscription::Table)
                    .name("idx_guild_subscription_sku_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(GuildSubscription::Table)
                    .name("idx_guild_subscription_guild_id_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(GuildData::Table)
                    .name("idx_guild_data_updated_at_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(GuildData::Table)
                    .name("idx_guild_data_guild_name_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(RegisteredUser::Table)
                    .name("idx_registered_user_anilist_id_unique")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
