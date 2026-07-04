//! Rolled-up vocal stats per (user_id, channel_id).
//!
//! Populated by the DB cleanup task when raw `vocal` rows are pruned, so that
//! XP totals derived from voice activity remain stable across the retention
//! cutoff. Reads should sum these archived values together with the surviving
//! raw `vocal` rows.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "vocal_summary")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub user_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub channel_id: String,
	pub session_count: i64,
	pub duration_total_seconds: i64,
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
