//! Rolled-up message counts per (user_id, channel_id).
//!
//! Populated by the DB cleanup task when raw `message` rows are pruned, so
//! that XP totals derived from message activity remain stable across the
//! retention cutoff.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "message_summary")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub user_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub channel_id: String,
	pub message_count: i64,
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
