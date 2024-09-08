//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_data")]

pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub username: String,
    pub is_bot: bool,
    pub banner: String,
    pub added_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]

pub enum Relation {
    #[sea_orm(has_many = "super::guild_subscription::Entity")]
    GuildSubscription,
    #[sea_orm(has_one = "super::registered_user::Entity")]
    RegisteredUser,
    #[sea_orm(has_many = "super::server_user_relation::Entity")]
    ServerUserRelation,
    #[sea_orm(has_one = "super::user_color::Entity")]
    UserColor,
    #[sea_orm(has_many = "super::user_subscription::Entity")]
    UserSubscription,
}

impl Related<super::guild_subscription::Entity> for Entity {
    fn to() -> RelationDef {

        Relation::GuildSubscription.def()
    }
}

impl Related<super::registered_user::Entity> for Entity {
    fn to() -> RelationDef {

        Relation::RegisteredUser.def()
    }
}

impl Related<super::server_user_relation::Entity> for Entity {
    fn to() -> RelationDef {

        Relation::ServerUserRelation.def()
    }
}

impl Related<super::user_color::Entity> for Entity {
    fn to() -> RelationDef {

        Relation::UserColor.def()
    }
}

impl Related<super::user_subscription::Entity> for Entity {
    fn to() -> RelationDef {

        Relation::UserSubscription.def()
    }
}

impl Related<super::guild_data::Entity> for Entity {
    fn to() -> RelationDef {

        super::server_user_relation::Relation::GuildData.def()
    }

    fn via() -> Option<RelationDef> {

        Some(super::server_user_relation::Relation::UserData.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]

pub enum RelatedEntity {
    #[sea_orm(entity = "super::guild_subscription::Entity")]
    GuildSubscription,
    #[sea_orm(entity = "super::registered_user::Entity")]
    RegisteredUser,
    #[sea_orm(entity = "super::server_user_relation::Entity")]
    ServerUserRelation,
    #[sea_orm(entity = "super::user_color::Entity")]
    UserColor,
    #[sea_orm(entity = "super::user_subscription::Entity")]
    UserSubscription,
    #[sea_orm(entity = "super::guild_data::Entity")]
    GuildData,
}
