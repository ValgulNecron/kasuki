//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "registered_user")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub user_id: String,
	pub anilist_id: i32,
	pub registered_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::user_data::Entity",
		from = "Column::UserId",
		to = "super::user_data::Column::UserId",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	UserData,
}

impl Related<super::user_data::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::UserData.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
