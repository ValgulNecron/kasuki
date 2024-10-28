//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "server_image")]

pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub server_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub image_type: String,
	pub server_name: String,
	pub image: String,
	pub image_url: String,
	pub generated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]

pub enum Relation {
	#[sea_orm(
		belongs_to = "super::guild_data::Entity",
		from = "Column::ServerId",
		to = "super::guild_data::Column::GuildId",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	GuildData,
}

impl Related<super::guild_data::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::GuildData.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]

pub enum RelatedEntity {
	#[sea_orm(entity = "super::guild_data::Entity")]
	GuildData,
}
