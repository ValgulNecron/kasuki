use sea_orm_migration::{prelude::*, schema::*};
use crate::m20240815_190656_module_activation::ModuleActivation;
use crate::m20240815_231355_kill_switch::KillSwitch;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove NewMembersModule from ModuleActivation
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleActivation::Table)
                    .drop_column(ModuleActivation::NewMembersModule)
                    .to_owned(),
            )
            .await?;

        // Add LevelModule and MiniGameModule to ModuleActivation
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleActivation::Table)
                    .add_column(boolean(Alias::new("LevelModule")).default(false))
                    .add_column(boolean(Alias::new("MiniGameModule")).default(false))
                    .to_owned(),
            )
            .await?;

        // Remove NewMembersModule from KillSwitch
        manager
            .alter_table(
                Table::alter()
                    .table(KillSwitch::Table)
                    .drop_column(KillSwitch::NewMembersModule)
                    .to_owned(),
            )
            .await?;

        // Add LevelModule and MiniGameModule to KillSwitch
        manager
            .alter_table(
                Table::alter()
                    .table(KillSwitch::Table)
                    .add_column(boolean(Alias::new("LevelModule")).default(false))
                    .add_column(boolean(Alias::new("MiniGameModule")).default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add back NewMembersModule to ModuleActivation
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleActivation::Table)
                    .add_column(boolean(Alias::new("NewMembersModule")).default(false))
                    .to_owned(),
            )
            .await?;

        // Remove LevelModule and MiniGameModule from ModuleActivation
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleActivation::Table)
                    .drop_column(Alias::new("LevelModule"))
                    .drop_column(Alias::new("MiniGameModule"))
                    .to_owned(),
            )
            .await?;

        // Add back NewMembersModule to KillSwitch
        manager
            .alter_table(
                Table::alter()
                    .table(KillSwitch::Table)
                    .add_column(boolean(Alias::new("NewMembersModule")).default(false))
                    .to_owned(),
            )
            .await?;

        // Remove LevelModule and MiniGameModule from KillSwitch
        manager
            .alter_table(
                Table::alter()
                    .table(KillSwitch::Table)
                    .drop_column(Alias::new("LevelModule"))
                    .drop_column(Alias::new("MiniGameModule"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
